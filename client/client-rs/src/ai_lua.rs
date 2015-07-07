extern crate lua;
use self::lua::ffi::lua_State;
//use self::lua;

extern crate libc;
use self::libc::c_int;

use salem::state::State;

use ai::Ai;

const WAIT: i64 = 1;
const GO  : i64 = 2;
const QUIT: i64 = 3;

//TODO FIXME ??? maybe it's possible to pass the salem::State handler here inside the lua_State struct?
//TODO FIXME     maybe even have to modificate lua_State struct
unsafe extern "C" fn test_c_callback (l: *mut lua_State) -> c_int {
    let mut lua = lua::State::from_ptr(l);
    let x = lua.to_integer(1);
    let y = lua.to_integer(2);
    println!("LUA: go: ({},{})", x, y);
    lua.push("state");
    lua.get_table(lua::REGISTRYINDEX);
    let addr = lua.to_integer(-1);
    println!("LUA: go: we received '{}'", addr);
    0
}

unsafe extern "C" fn out (l: *mut lua_State) -> c_int {
    let mut lua = lua::State::from_ptr(l);
    //TODO check arguments count on stack
    //TODO match type_of(1) { Table => print rows }
    match lua.to_str(1) {
        Some(s) => { println!("LUA: out: {}", s); }
        None    => { println!("LUA: out: {:?}", lua.type_of(1)); }
    }
    0
}

pub struct LuaAi {
    lua: /*&'b mut*/ lua::State
}

impl LuaAi {
    pub fn new () -> LuaAi {
        LuaAi {lua : lua::State::new()}
    }

    /*
    fn stack_dump (&mut self) {
        let top = self.lua.get_top();
        for i in 1..top+1 {
            print!("{}={:?} ", i, self.lua.type_of(i));
        }
        println!("");
    }
    */

    fn stack_len (&mut self) -> i32 {
        self.lua.get_top()
    }

    fn stack_is_empty (&mut self) -> bool {
        self.stack_len() == 0
    }

    fn check_stack (&mut self, note: &str) {
        if !self.stack_is_empty() { panic!("{}: stack is NOT empty", note); }
    }

    fn check_status (&mut self, note: &str) {
        if self.lua.status() != lua::ThreadStatus::Ok { panic!("{}: Lua status is NOT ok", note); }
    }

    fn check (&mut self, note: &str) {
        self.check_status(note);
        self.check_stack(note);
    }

    //TODO use Cow to possibility pass String or &str
    fn exec (&mut self, string: &str) {
        let status = self.lua.do_string(string);
        match status {
            lua::ThreadStatus::Ok => {
                //println!("do_string: ok\n")
            }
            _ => {
                match self.lua.to_type::<String>() {
                    Some(s) => { self.lua.pop(1); println!("lua: {:?}: {}\n", status, s); }
                    None    => { println!("lua: {:?}: no info\n", status); }
                }
            }
        }
    }

    //XXX ??? return Option<i64> ?
    fn get_number (&mut self, string: &str) -> i64 {
        let t = self.lua.get_global(string);
        if t != lua::Type::Number {
            self.lua.pop(-1);
            panic!("{}: is NOT a number", string);
        }
        let ret = self.lua.to_integer(-1);
        self.lua.pop(-1);
        ret
    }

    fn set_number (&mut self, string: &str, number: i64) {
        self.lua.push_integer(number);
        self.lua.set_global(string);
        self.check("set_number");
    }

    pub fn init (&mut self) {
        //TODO FIXME re-direct all output from stdio to user TCP connection
        //TODO FIXME XXX somehow pass client context to lua context
        //               to call client context methods within callbacks

        self.lua.open_libs();

        self.lua.push_string("client");
        self.lua.push_integer(42);
        self.lua.set_table(lua::REGISTRYINDEX);
        
        self.lua.push_fn(Some(test_c_callback));
        self.lua.set_global("test_c_callback");
        
        self.lua.push_fn(Some(out));
        self.lua.set_global("out");
        
        self.lua.push_integer(WAIT);
        self.lua.set_global("WAIT");
        
        self.lua.push_integer(GO);
        self.lua.set_global("GO");
        
        self.lua.push_integer(QUIT);
        self.lua.set_global("QUIT");

        self.lua.new_table();         // stack: table(1,-1)
        self.lua.push_integer(0);     // stack: table(1,-2) key:int(2,-1)
        self.lua.push_string("root"); // stack: table(1,-3) key:int(2,-2) value:string(3,-1)
        self.lua.set_table(1);        // table[key] = value, stack: table(-1)
        self.lua.set_global("widgets");

        //TODO wrap to fn lua_do (&str) -> String { ... }
        //TODO load lua code from file
        //FIXME we have to wait all game widgets are added (mapview) before start co-routine
        self.exec("
            g_action = 0

            function wait_100msec ()
                out('wait')
                g_action = WAIT
                coroutine.yield()
            end

            wait = function (decisec)
                out('wait ' .. decisec .. ' decisec')
                while decisec > 0 do
                    decisec = decisec - 1
                    wait_100msec()
                end
            end
            
            go = function (x, y)
                out('go (' .. x .. ',' .. y .. ')')
                g_action = GO
                g_x = x
                g_y = y
                while not hero_is_walking do
                    -- out('hero is NOT start walking')
                    coroutine.yield()
                end
                while hero_is_walking do
                    -- out('hero is STILL walking')
                    coroutine.yield()
                end
            end

            go_rel = function (x, y)
                out('go_rel (' .. x .. ',' .. y .. ')')
                go(hero_x + x, hero_y + y)
            end
            
            quit = function ()
                out('quit')
                g_action = QUIT
            end

            widget_id = function (name)
                for wid,wname in pairs(widgets) do
                    if wname == name then
                        return wid
                    end
                end
                return nil
            end

            widget_exists = function (name)
                return (not (widget_id(name) == nil))
            end

            wait_while_char_enters_game = function ()
                out('wait_while_char_enters_game')
                while not widget_exists('mapview') do
                    out('widgets[mapview] == nil')
                    coroutine.yield()
                end
                while hero_x == nil or hero_y == nil do
                    out('hero_x or hero_y == nil')
                    coroutine.yield()
                end
                while hero_grid == nil do
                    out('hero_grid == nil')
                    coroutine.yield()
                end
            end
            
            main = function ()
                wait_while_char_enters_game()
                -- TODO while user_script == nil do yield end user_script()
                local run = true
                while run == true do
                    go_rel(-100,0)
                    wait(10)
                    go_rel(0,100)
                    wait(10)
                    go_rel(100,0)
                    wait(10)
                    go_rel(0,-100)
                    wait(10)
                end
            end
            
            co = coroutine.create(main)
        ");
    }
    
    fn update_environment (&mut self, state: &State) {
        self.check("UPDATE");

        //let wtype = self.lua.get_global("widgets");
        //if wtype != lua::Type::Table {
        //    println!("ERROR: widgets type({:?}) is not a Table", wtype);
        //    return;
        //}
        self.lua.new_table();
        for w in state.widgets.values() {
            self.lua.push(w.id as i64);
            self.lua.push(w.typ.as_str());
            self.lua.set_table(1);
            //self.lua.push_string(w.typ.as_str()); // stack: table(-2) value:string(-1)
            //self.lua.raw_seti(-2, w.id as i64);       // table[key] = value (table[0] = "root"), stack: table(-1)
        }
        self.lua.set_global("widgets");
        //self.lua.pop(1);

        match state.hero_xy() {
            Some((x,y)) => {
                self.lua.push(x as i64);
                self.lua.set_global("hero_x");
                self.lua.push(y as i64);
                self.lua.set_global("hero_y");
            }
            None => {
                self.lua.push_nil();
                self.lua.set_global("hero_x");
                self.lua.push_nil();
                self.lua.set_global("hero_y");
            }
        }

        //TODO pass whole grid Z coords to Lua environment
        match state.hero_grid() {
            Some(_) => {
                self.lua.push(1);
                self.lua.set_global("hero_grid");
            }
            None => {
                self.lua.push_nil();
                self.lua.set_global("hero_grid");
            }
        }

        match state.hero_obj() {
            Some(hero) => {
                match hero.movement {
                    Some(_) => { self.lua.push_bool(true); }
                    None    => { self.lua.push_bool(false); }
                }
            }
            None => {
                self.lua.push_bool(false);
            }
        }
        self.lua.set_global("hero_is_walking");

        self.check("UPDATE");
    }

    fn react (&mut self) {
        self.check("REACT");
        self.exec("coroutine.resume(co)");
        self.check("REACT");
    }
    
    fn dispatch_pendings (&mut self, state: &mut State) {
        self.check("DISPATCH");
        
        let action = self.get_number("g_action");

        match action {
            QUIT => {
                println!("QUIT");
                match state.close() {
                    Ok(_) => {}
                    Err(e) => { println!("ERROR: state.close: {:?}", e); }
                }
            }
            GO => {
                println!("GO");

                let x = self.get_number("g_x");
                let y = self.get_number("g_y");
                
                if let Err(e) = state.go(x as i32, y as i32) {
                    println!("ERROR: state.go: {:?}", e);
                }
            }
            _ => {
                //println!("???: {}", action);
            }
        }
        self.set_number("g_action", 0);

        self.check("DISPATCH");
    }
}

impl Ai for LuaAi {
    fn update (&mut self, state: &mut State) {
        self.update_environment(state);
        self.react();
        self.dispatch_pendings(state);
    }
    
    fn exec (&mut self, s: &str) {
        self.exec(s);
    }
    
    fn init (&mut self) {
        self.init();
        println!("Lua AI initialised");
    }
    
    fn new () -> LuaAi {
        Self::new()
    }
}

