import socket, ssl, hashlib, time, threading, struct, sys
from direct.showbase.ShowBase import ShowBase
from panda3d.core import *
from direct.distributed import PyDatagram, PyDatagramIterator
from direct.task import *
from direct.interval.IntervalGlobal import *
import logging
from pandac.PandaModules import loadPrcFileData
from direct.gui.DirectGui import *
import os


def dbg(data):
	logging.info(data)


class Struct:
	def __init__(self, **kwds):
		self.__dict__.update(kwds)


msg_type = Struct(SESS=0, REL=1, ACK=2, BEAT=3, MAPREQ=4, MAPDATA=5, OBJDATA=6, OBJACK=7, CLOSE=8)
arg_type = Struct(END=0, INT=1, STR=2, COORD=3, COLOR=6)
rel_type = Struct(NEWWDG=0, WDGMSG=1, DSTWDG=2, MAPIV=3, GLOBLOB=4, PAGINAE=5, RESID=6, PARTY=7, SFX=8, CATTR=9, MUSIC=10, TILES=11, BUFF=12)

#TODO: config = Struct(beat_interval=???, ...)

#TODO tx queue concept:
#		que = [(timeout, last_sent, type, seq, datagram) ... ()]
#		que.add_to_front() - maybe
#		que.add_to_back() - maybe
#		maybe use priorities mechanics?
#			BEAT - has lowest priority
#		BEAT - is always the last unremovable (because of unACKable) item on que
#
#	class tx_que:
#		def add_req(timeout, last_sent, type, seq, datagram):
#			self.que[0:0] = tx_req(timeout, last_sent, type, seq, datagram)
#
#	class tx_req:
#		...
#
#	delta = task.time - que[0].last_sent
#	if delta > que[0].timeout:
#		que[0].last_sent = task.time

# class txQueue():
	# def __init__(self):
		# self.ack_que = []
		# self.que = []

	# def add(self):
		# request = Struct(timeout, last_sent, type, seq, datagram)
		# self.que[???] = 

	# def add_sess(self):
		# dbg('---> add SESS')
		# self.add(10., msg_type.BEAT, 0, self.make_beat())

	# def add_beat(self):
		# dbg('---> add BEAT')
		# self.add(10., msg_type.BEAT, 0, self.make_beat())

	# def add_ack(self):
		# self.ack_que.append(...)

	# def remove_sess(self):
		# dbg('---> remove all SESS')
		# self.que = [i for i in self.que if i.type != msg_type.SESS]


class hnh_client(ShowBase):
	def __init__(self, host, ssl_port, udp_port):
		self.host = host
		self.ssl_port = ssl_port
		self.udp_port = udp_port
		self.addr = NetAddress()
		self.addr.setHost(self.host, self.udp_port)
		self.tiles = {}
		self.widgets = {}
		self.resources = {}
		self.chars = {}
		self.tx_que = []
		self.tx_seq = 0
		if os.name == 'posix':
			logging.basicConfig(format='%(asctime)s  %(message)s', level=logging.INFO)
		else:
			logging.basicConfig(filename='client.log', filemode="w", format='%(asctime)s  %(message)s', level=logging.INFO)
		self.new_widget(0, 'ui_root', (0,0), None, [])
		self.sess_errors = {
			0:'OK',
			1:'AUTH',
			2:'BUSY',
			3:'CONN',
			4:'PVER',
			5:'EXPR'
		}
		self.msg_types = {
			0:('SESS', self.rx_sess),
			1:('REL', self.rx_rel),
			2:('ACK', self.rx_ack),
			3:('BEAT', self.rx_beat),
			4:('MAPREQ', self.rx_mapreq),
			5:('MAPDATA', self.rx_mapdata),
			6:('OBJDATA', self.rx_objdata),
			7:('OBJACK', self.rx_objack),
			8:('CLOSE', self.rx_close)
		}
		self.rel_types = {
			0:('NEWWDG', self.rx_rel_newwdg),
			1:('WDGMSG', self.rx_rel_wdgmsg),
			2:('DSTWDG', self.rx_rel_dstwdg),
			3:('MAPIV', self.rx_rel_mapiv),
			4:('GLOBLOB', self.rx_rel_globlob),
			5:('PAGINAE', self.rx_rel_paginae),
			6:('RESID', self.rx_rel_resid),
			7:('PARTY', self.rx_rel_party),
			8:('SFX', self.rx_rel_sfx),
			9:('CATTR', self.rx_rel_cattr),
			10:('MUSIC', self.rx_rel_music),
			11:('TILES', self.rx_rel_tiles),
			12:('BUFF', self.rx_rel_buff)
		}
		self.wdg_list_types = {
			0:'END',
			1:'INT',
			2:'STR',
			3:'COORD',
			6:'COLOR'
		}

	def authorize(self, name, password):
		# try:
			# f = open('cookie', 'rb')
			# self.cookie = f.read()
			# dbg('using cached cookie')
		# except:
		s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
		ss = ssl.wrap_socket(s)
		ss.connect((self.host, self.ssl_port))
		msg = bytes(bytearray([1,len(name)])+name.encode('utf8'))
		ss.write(msg)
		msg = ss.read(2)
		msg_type, length = struct.unpack('!BB', msg)
		if length > 0:
			msg = ss.read(length)
		if(msg_type != 0):
			dbg('username binding: wrong message type "'+str(msg_type)+'" '+msg)
			ss.close()
			return False
		hash = hashlib.sha256()
		hash.update(password.encode('utf8'))
		hash = hash.digest()
		msg = bytes(bytearray([2,len(hash)])+hash)
		ss.write(msg)
		msg = ss.read(2)
		msg_type, length = struct.unpack('!BB', msg)
		if length > 0:
			msg = ss.read(length)
		ss.close()
		if(msg_type != 0):
			dbg('password binding: wrong message type "'+str(msg_type)+'" '+msg)
			return False
		self.cookie = msg
		#f = open('cookie','wb')
		#f.write(self.cookie)
		#f.close()
		self.user = name
		#dbg('cookie: '+self.cookie)
		return True

	def start(self):
		loadPrcFileData("", "window-title HNH")
		loadPrcFileData("", "fullscreen 0")
		loadPrcFileData("", "win-size 400 400")
		loadPrcFileData("", "win-origin 40 50")
		# FOR REALTIME win props changing:
		# wp = WindowProperties() 
		# base.win.requestProperties(wp)
		# wp.clearSize()
		# wp.setSize(100, 100)
		# wp.setTitle('Test')
		# for fullscreen see http://www.panda3d.org/forums/viewtopic.php?t=2848
		# for fullscreen see http://www.panda3d.org/forums/viewtopic.php?t=6105
		ShowBase.__init__(self)
		self.setFrameRateMeter(True)
		self.accept("escape", sys.exit)
		self.cmanager = QueuedConnectionManager()
		self.creader = QueuedConnectionReader(self.cmanager, 0)
		self.creader.setRawMode(True)
		self.cwriter = ConnectionWriter(self.cmanager, 0)
		self.cwriter.setRawMode(True)
		self.conn = self.cmanager.openUDPConnection(self.udp_port)
		if not self.conn:
			dbg("failed to create connection")
			return
		self.conn.setReuseAddr(True)
		self.creader.addConnection(self.conn)
		taskMgr.add(self.rx_task, "rx_task")
		taskMgr.add(self.tx_task, "tx_task")
		self.tx_add_sess()
		#TODO fsm.enter(sess)
		self.last_sent = -10.
		self.run()

	######################################################  RX  #####################################

	def rx_task(self, task):
		if self.creader.dataAvailable():
			datagram = NetDatagram()
			if self.creader.getData(datagram):
				data = PyDatagramIterator.PyDatagramIterator(datagram)
				msg_type = data.getUint8()
				if msg_type not in self.msg_types:
					dbg("UNKNOWN PACKET TYPE "+str(msg_type))
				else:
					dbg(self.msg_types[msg_type][0])
					self.msg_types[msg_type][1](data)
		return task.cont

	def rx_sess(self, data):
		error = data.getUint8()
		if error == 0:
			self.tx_remove_sess()
		if error in self.sess_errors:
			dbg('  error={0} ({1})'.format(error, self.sess_errors[error]))
		else:
			dbg('  error={0} (unknown)'.format(error))

	def rx_rel(self, data):
		seq = data.getUint16()
		while data.getRemainingSize():
			rel_type = data.getUint8()
			if rel_type&0x80 != 0:
				rel_type &= 0x7f;
				rel_len = data.getUint16()
			else:
				rel_len = data.getRemainingSize()
			if rel_type not in self.rel_types:
				dbg('  seq={0} rel=UNKNOWN ({1}) len={2}'.format(seq, rel_type, rel_len))
				data.skipBytes(rel_len)
			else:
				dbg('  seq={0} rel={1} len={2}'.format(seq, self.rel_types[rel_type][0], rel_len))
				rel = data.extractBytes(rel_len)
				dg = Datagram(rel)
				pdi = PyDatagramIterator.PyDatagramIterator(dg)
				self.rel_types[rel_type][1](pdi)
			seq += 1
		self.tx_ask(seq-1)

	def rx_rel_parse_args(self, data):
		args = []
		while data.getRemainingSize():
			arg_type = data.getUint8()
			if arg_type == 0: # END
				dbg('      END')
				if data.getRemainingSize():
					dbg('      DATA REMAINS')
			elif arg_type == 1: # INT
				args.append(Struct(type=arg_type, value=data.getInt32()))
			elif arg_type == 2: # STR
				args.append(Struct(type=arg_type, value=data.getZString()))
			elif arg_type == 3: # COORD
				args.append(Struct(type=arg_type, value=(data.getInt32(), data.getInt32())))
			elif arg_type == 6: # COLOR
				args.append(Struct(type=arg_type, value=(data.getUint8(), data.getUint8(), data.getUint8(), data.getUint8())))
			else: # UNKNOWN LIST TYPE
				break
		return args
	
	def rx_rel_newwdg(self, data):
		wdg_id = data.getUint16()
		wdg_type = data.getZString()
		wdg_coord = (data.getInt32(),data.getInt32())
		wdg_parent = data.getUint16()
		wdg_args = self.rx_rel_parse_args(data)
		self.new_widget(wdg_id, wdg_type, wdg_coord, wdg_parent, wdg_args)

	def rx_rel_wdgmsg(self, data):
		wdg_id = data.getUint16()
		wdg_msg_name = data.getZString()
		wdg_args = self.rx_rel_parse_args(data)
		self.widget_message(wdg_id, wdg_msg_name, wdg_args)

	def rx_rel_dstwdg(self, data):
		wdg_id = data.getUint16()
		self.destroy_widget(wdg_id)
		dbg('    id={0}'.format(wdg_id))

	def rx_rel_mapiv(self, data):
		pass

	def rx_rel_globlob(self, data):
		pass

	def rx_rel_paginae(self, data):
		pass

	def rx_rel_resid(self, data):
		res_id = data.getUint16()
		res_name = data.getZString()
		res_ver = data.getUint16()
		self.resources[res_id] = Struct(name=res_name, version=res_ver)
		# dbg('    id={0} name={1} ver={2}'.format(res_id, res_name, res_ver))

	def rx_rel_party(self, data):
		pass

	def rx_rel_sfx(self, data):
		pass

	def rx_rel_cattr(self, data):
		pass

	def rx_rel_music(self, data):
		pass

	def rx_rel_tiles(self, data):
		while data.getRemainingSize():
			tile_id = data.getUint8()
			tile_name = data.getZString()
			tile_ver = data.getUint16()
			self.tiles[tile_id] = (tile_name, tile_ver)
			dbg('    id={0} name={1} version={2}'.format(tile_id,tile_name,tile_ver))

	def rx_rel_buff(self, data):
		pass

	def rx_ack(self, data):
		seq = data.getUint16()
		if self.tx_que[0].seq == seq:
			req = self.tx_que[0]
			self.tx_que = self.tx_que[1:]
			dbg('  TXQUE: removed {0} seq={1}'.format(self.msg_types[req.type][0], req.seq))
		dbg('  seq={0}'.format(seq))

	def rx_beat(self, data):
		pass

	def rx_mapreq(self, data):
		pass

	def rx_mapdata(self, data):
		pass

	def rx_objdata(self, data):
		pass

	def rx_objack(self, data):
		pass

	def rx_close(self, data):
		pass

	######################################################  TX  #####################################

	def tx_task(self, task):
		# for req in self.tx_que.ack_que:
			# self.cwriter.send(req.data, self.conn, self.addr)
		# self.tx_que.ack_que = []
		#TODO calculate the delta taking into account time counter overflow
		delta = task.time - self.last_sent
		if len(self.tx_que) > 0:
			req = self.tx_que[0]
		else:
			req = self.tx_make_beat()
		if delta > req.timeout:
			self.cwriter.send(req.data, self.conn, self.addr)
			dbg('---> {0} seq={1}'.format(self.msg_types[req.type][0], req.seq))
			self.last_sent = task.time
		return task.cont

	def tx_add_sess(self):
		data = PyDatagram.PyDatagram()
		data.addUint8(0) # SESS
		data.addUint16(1) # ???
		data.addZString(u'Haven') # protocol name
		data.addUint16(2) # version
		data.addZString(self.user)
		data.appendData(self.cookie)
		# self.cwriter.send(data, self.conn, self.addr)
		self.tx_que.append(Struct(timeout=1, type=msg_type.SESS, seq=0, data=data))

	def tx_remove_sess(self):
		self.tx_que = [i for i in self.tx_que if i.type != msg_type.SESS]
		dbg('---> all SESS removed')

	def tx_add_rel_wdgmsg(self, seq, wdg_id, msg_name, args):
		data = PyDatagram.PyDatagram()
		data.addUint8(msg_type.REL)
		data.addUint16(seq)
		data.addUint8(rel_type.WDGMSG)
		data.addUint16(wdg_id)
		data.addZString(msg_name)
		for arg in args:
			if arg.type == arg_type.END:
				data.addUint8(arg.type)
			elif arg.type == arg_type.INT:
				data.addUint8(arg.type)
				data.addInt32(arg.value)
			elif arg.type == arg_type.STR:
				data.addUint8(arg.type)
				data.addZString(arg.value)
			elif arg.type == arg_type.COORD:
				data.addUint8(arg.type)
				data.addInt32(arg.value[0])
				data.addInt32(arg.value[1])
			elif arg.type == arg_type.COLOR:
				data.addUint8(arg.type)
				data.addUint8(arg.value[0])
				data.addUint8(arg.value[1])
				data.addUint8(arg.value[2])
				data.addUint8(arg.value[3])
			else:
				DBG('!!! UNKNOWN arg type {0}'.format(arg.type))
				return
		self.tx_que.append(Struct(timeout=.4, type=msg_type.REL, seq=seq, data=data))
		self.tx_seq += 1

	def tx_ask(self, seq):
		data = PyDatagram.PyDatagram()
		data.addUint8(msg_type.ACK)
		data.addUint16(seq)
		self.cwriter.send(data, self.conn, self.addr)
		dbg("---> ACK seq={0}".format(seq))

	def tx_make_beat(self):
		data = PyDatagram.PyDatagram()
		data.addUint8(msg_type.BEAT)
		return Struct(timeout=10, type=msg_type.BEAT, seq=0, data=data)

	def new_widget(self, wdg_id, wdg_type, wdg_coord, wdg_parent, wdg_args):
		dbg('    id={0} type={1} coord={2} parent={3}'.format(wdg_id, wdg_type, wdg_coord, wdg_parent))
		self.widgets[wdg_id] = Struct(type=wdg_type, coord=wdg_coord, parent=wdg_parent, args=wdg_args)
		for arg in wdg_args:
			dbg('      {0}={1}'.format(self.wdg_list_types[arg.type], arg.value))

	def widget_message(self, wdg_id, wdg_msg_name, wdg_args):
		dbg('    id={0} name={1}'.format(wdg_id, wdg_msg_name))
		if (self.widgets[wdg_id].type == 'charlist') and (wdg_msg_name == 'add'):
			if wdg_args[0].value not in self.chars:
				char = Struct(name=wdg_args[0].value, equip=[arg.value for arg in wdg_args[1:]])
				b = DirectButton(text = (char.name), scale=.05, pos=(-.9,0,.9-.1*len(self.chars)), command=self.choice_char, extraArgs=[char])
				self.chars[char.name] = char
				dbg('      add character: name={0} equipment:'.format(char.name))
				for equip in char.equip:
					dbg('                    {0} ver={1}'.format(self.resources[equip].name, self.resources[equip].version))
			else:
				dbg('      character "{0}" is already added'.format(wdg_args[0].value))

	def destroy_widget(self, wdg_id):
		#TODO
		pass
	
	def choice_char(self, char):
		dbg('SELECT "{0}"'.format(char.name))
		self.tx_add_rel_wdgmsg(seq=self.tx_seq, wdg_id=4, msg_name='play', args=[Struct(type=arg_type.STR, value='first')])
		# CLIENT
		 # REL (1)
		  # seq=1 type=1(WDGMSG) len=14
		   # id=4 name=play
			# STR=first

###########################################################################

hnh = hnh_client('moltke.seatribe.se', 1871, 1870)
while not hnh.authorize(u'lemings', u'lemings'):
	dbg('authorization failed')
	#TODO add delay
dbg('authorized')
hnh.start()
#TODO
#	hnh.start() { wait for all 5 widgets of the first screen }
#	if hnh.chars.length() == 0:
#		hnh.create_new_char('lemingX')
#	hnh.choice_char(0)
#	if not hnh.enter_game_there_logoff():
#		hnh.enter_game_on_hf()


