import socket, ssl, hashlib, time, threading, struct, sys
from direct.showbase.ShowBase import ShowBase
from panda3d.core import *
from direct.distributed import PyDatagram, PyDatagramIterator
from direct.task import *
from direct.interval.IntervalGlobal import *
import logging
from pandac.PandaModules import loadPrcFileData
from direct.gui.DirectGui import *


#def dbg(data):
#	logging.info(data)

def dbg(data):
	print data


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
		logging.basicConfig(filename='client.log', filemode="w", level=logging.INFO)
		
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
		loadPrcFileData("", "win-size 900 900")
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
		self.current_request = self.tx_sess
		#TODO fsm.enter(sess)
		self.last_tx_time = -10.
		self.run()

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

	def tx_task(self, task):
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
		delta = task.time - self.last_tx_time
		if self.current_request != None:
			if delta > 1.0: #TODO - hardcoding
				self.current_request()
				self.last_tx_time = task.time
		else:
			if delta > 5.0:
				self.tx_beat()
				self.last_tx_time = task.time
		return task.cont

	def rx_sess(self, data):
		error = data.getUint8()
		if error == 0:
			self.current_request = None
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
				args.append((arg_type, data.getInt32()))
			elif arg_type == 2: # STR
				args.append((arg_type, data.getZString()))
			elif arg_type == 3: # COORD
				args.append((arg_type, (data.getInt32(), data.getInt32())))
			elif arg_type == 6: # COLOR
				args.append((arg_type, (data.getUint8(), data.getUint8(), data.getUint8(), data.getUint8())))
			else: # UNKNOWN LIST TYPE
				break
		return args
	
	def rx_rel_newwdg(self, data):
		wdg_id = data.getUint16()
		wdg_type = data.getZString()
		wdg_coord = [data.getInt32(),data.getInt32()]
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
		self.resources[res_id] = (res_name, res_ver)
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
		self.tx_que.ack(seq)

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

	def tx_sess(self):
		data = PyDatagram.PyDatagram()
		data.addUint8(0) # SESS
		data.addUint16(1) # ???
		data.addZString(u'Haven') # protocol name
		data.addUint16(2) # version
		data.addZString(self.user)
		data.appendData(self.cookie)
		self.cwriter.send(data, self.conn, self.addr)
		dbg("---> SESS")

#	def tx_rel_wdgmsg(self, seq):
#		data = PyDatagram.PyDatagram()
#		data.addUint8(1) # REL
#		data.addUint16(seq)
#		self.cwriter.send(data, self.conn, self.addr)

	def tx_ask(self, seq):
		data = PyDatagram.PyDatagram()
		data.addUint8(2) # ACK
		data.addUint16(seq)
		self.cwriter.send(data, self.conn, self.addr)
		dbg("---> ACK seq={0}".format(seq))

	def tx_beat(self):
		data = PyDatagram.PyDatagram()
		data.addUint8(3) # BEAT
		self.cwriter.send(data, self.conn, self.addr)
		dbg("---> BEAT")

	def new_widget(self, wdg_id, wdg_type, wdg_coord, wdg_parent, wdg_args):
		dbg('    id={0} type={1} coord={2} parent={3}'.format(wdg_id, wdg_type, wdg_coord, wdg_parent))
		if wdg_type == 'cnt':
			wdg = DirectFrame(frameColor=(0, 0, 0, .5), frameSize=(-1, 1, -1, 1), pos=(1, -1, -1))
		else:
			wdg = None
		self.widgets[wdg_id] = (wdg_type, wdg_parent, wdg)
		for arg in wdg_args:
			dbg('      {0}={1}'.format(self.wdg_list_types[arg[0]], arg[1]))

	def widget_message(self, wdg_id, wdg_msg_name, wdg_args):
		dbg('    id={0} name={1}'.format(wdg_id, wdg_msg_name))
		for arg in wdg_args:
			if wdg_msg_name == "add" and arg[0] == 1: # INT
				dbg('      {0}={1} ({2})'.format(self.wdg_list_types[arg[0]], self.resources[arg[1]][0], self.resources[arg[1]][1]))
			else:
				dbg('      {0}={1}'.format(self.wdg_list_types[arg[0]], arg[1]))

	def destroy_widget(self, wdg_id):
		#TODO
		pass
	
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


