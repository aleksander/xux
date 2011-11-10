import socket, ssl, hashlib, time, threading, struct, sys
from direct.showbase.ShowBase import ShowBase
from panda3d.core import *
from direct.distributed import PyDatagram, PyDatagramIterator
from direct.task import *
from direct.interval.IntervalGlobal import *

############################################################################

def dbg(data):
	print data

############################################################################

class txQueue:
	def __init__(self, conn, cwriter, addr):
		self.conn = conn
		self.cwriter = cwriter
		self.addr = addr
		self.que = [] # (seq, type, packet)
		self.last_tx_time = -10.
		
	#def ack(self):
	#	pass

	def add_sess(self, user, cookie):
		data = self.make_sess(user, cookie)
		self.que.append((0, 0, data))

	def remove_sess(self):
		dbg("remove sess")
		self.que = [i for i in self.que if i[1] != 0]
		if len(self.que) > 0:
			self.last_tx_time = -10
		
	#def add(self, seq, packet):
	#	pass

	def make_sess(self, user, cookie):
		data = PyDatagram.PyDatagram()
		data.addUint8(0) # SESS
		data.addUint16(1) # ???
		data.addZString(u'Haven') # protocol name
		data.addUint16(2) # version
		data.addZString(user)
		data.appendData(cookie)
		return data

	def send_ask(self, seq):
		data = PyDatagram.PyDatagram()
		data.addUint8(2) # ACK
		data.addUint16(seq)
		self.cwriter.send(data, self.conn, self.addr)

	def serve(self, time):
		delta = time - self.last_tx_time
		if delta > 1.0: #TODO - hardcoding
			if len(self.que) > 0:
				self.cwriter.send(self.que[0][2], self.conn, self.addr)
				dbg("sent. delta={0}".format(delta))
			else:
				#TODO send BEAT
				pass
			self.last_tx_time = time
		# send_current_request() until not current_datagram_acked()
		#if current_request not acked():
		#	if curren_time() - last_send_time > 0.1:
		#		send(current_request)
		#		last_send_time = current_time()
		#if curren_time() - last_send_time > 1.:
		#	send(BEAT)

class hnh_client(ShowBase):
	def __init__(self, host, ssl_port, udp_port):
		self.host = host
		self.ssl_port = ssl_port
		self.udp_port = udp_port
		self.addr = NetAddress()
		self.addr.setHost(self.host, self.udp_port)
		self.tiles = {}
		self.widgets = []
		#for i in range(0,256):
		#	self.tiles.append('')
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
		self.user_name = name
		#dbg('cookie: '+self.cookie)
		return True

	def start(self):
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
		self.tx_que = txQueue(self.conn, self.cwriter, self.addr)
		self.tx_que.add_sess(self.user_name, self.cookie)
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
		self.tx_que.serve(task.time)
		return task.cont

	def rx_sess(self, data):
		error = data.getUint8()
		if error == 0:
			self.tx_que.remove_sess()
		if error in sess_errors:
			dbg('  error={0} ({1})'.format(error, sess_errors[error]))
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
				dbg('  seq={0} rel={0} len={2}'.format(seq, self.rel_types[rel_type][0], rel_len))
				rel = data.extractBytes(rel_len)
				dg = Datagram(rel)
				pdi = PyDatagramIterator.PyDatagramIterator(dg)
				self.rel_types[rel_type][1](pdi)
			seq += 1
		self.tx_que.send_ask(seq)

	def rx_rel_newwdg(self, data):
		wdg_id = data.getUint16()
		wdg_type = data.getZString()
		wdg_coord = [data.getInt32(),data.getInt32()]
		wdg_parent = data.getUint16()
		self.new_widget(wdg_id, wdg_type, wdg_parent)
		dbg('    id={0} type={1} coord={2} parent={3}'.format(wdg_id,wdg_type,wdg_coord,wdg_parent))
		while data.getRemainingSize():
			wdg_lt = data.getUint8()
			if wdg_lt not in self.wdg_list_types:
				dbg('     UNKNOWN LIST TYPE')
				break
			if wdg_lt == 0: # END
				dbg('      END')
			elif wdg_lt == 1: # INT
				dbg('      {0}={1}'.format(self.wdg_list_types[wdg_lt], data.getInt32()))
			elif wdg_lt == 2: # STR
				dbg('      {0}={1}'.format(self.wdg_list_types[wdg_lt], data.getZString()))
			elif wdg_lt == 3: # COORD
				dbg('      {0}={1}'.format(self.wdg_list_types[wdg_lt], [data.getInt32(),data.getInt32()]))
			elif wdg_lt == 6: # COLOR
				dbg('      {0}={1}'.format(self.wdg_list_types[wdg_lt], [data.getUint8(),data.getUint8(),data.getUint8(),data.getUint8()]))

	def rx_rel_wdgmsg(self, data):
		wdg_id = data.getUint16()
		wdg_msg_name = data.getZString()
		dbg('    id={0} name={1}'.format(wdg_id, wdg_msg_name))
		while data.getRemainingSize():
			wdg_lt = data.getUint8()
			if wdg_lt not in self.wdg_list_types:
				dbg('     UNKNOWN LIST TYPE')
				break
			if wdg_lt == 0: # END
				dbg('      END')
			elif wdg_lt == 1: # INT
				dbg('      {0}={1}'.format(self.wdg_list_types[wdg_lt], data.getInt32()))
			elif wdg_lt == 2: # STR
				dbg('      {0}={1}'.format(self.wdg_list_types[wdg_lt], data.getZString()))
			elif wdg_lt == 3: # COORD
				dbg('      {0}={1}'.format(self.wdg_list_types[wdg_lt], [data.getInt32(),data.getInt32()]))
			elif wdg_lt == 6: # COLOR
				dbg('      {0}={1}'.format(self.wdg_list_types[wdg_lt], [data.getUint8(),data.getUint8(),data.getUint8(),data.getUint8()]))

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
		pass

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

#	def tx_rel_wdgmsg(self, seq):
#		data = PyDatagram.PyDatagram()
#		data.addUint8(1) # REL
#		data.addUint16(seq)
#		self.cwriter.send(data, self.conn, self.addr)
		
	def new_widget(self, wdg_id, wdg_type, parent, args = []):
		# self.widgets[wdg_id] = (wdg_type, parent)
		pass

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


