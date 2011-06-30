import socket, ssl, hashlib, time, threading, struct
from direct.showbase.ShowBase import ShowBase
from panda3d.core import *
from direct.distributed import PyDatagram, PyDatagramIterator
from direct.task import *

#####################################################################

#def show_bytes(s, msg):
#	print(s, end='')
#	for b in msg:
#		print('{0:02X} '.format(b),end='')
#	print('')

############################################################################

#class message:
#	def __init__(self, b):
#		self.type = b[0]
#		self.len = b[1]
#		self.body = b[2:]
#	def recv(self, s):
#		self.type = s.recv(1)
#		self.len = s.recv(1)
#		self.body = s.recv(self.len)
#		return self
#	def send(self, s):
#		return self
#	def deliver(self, s):
#		s.send(self.type)
#		s.send(self.len)
#		s.send(self.body)
#		return self

###########################################################################################

def dbg(data):
	print data

############################################################################

class hnh_client(ShowBase):
	def __init__(self, host, ssl_port, udp_port):
		self.host = host
		self.ssl_port = ssl_port
		self.udp_port = udp_port
		self.addr = NetAddress()
		self.addr.setHost(self.host, self.udp_port)
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
		# f = open('cookie','wb')
		# f.write(self.cookie)
		# f.close()
		self.user_name = name
		dbg('cookie: '+self.cookie)
		return True

	def start(self):
		ShowBase.__init__(self)
		self.cmanager = QueuedConnectionManager()
		self.creader = QueuedConnectionReader(self.cmanager, 0)
		self.cwriter = ConnectionWriter(self.cmanager, 0)
		self.cwriter.setRawMode(True)
		self.creader.setRawMode(True)
		self.conn = self.cmanager.openUDPConnection(self.udp_port)
		if not self.conn:
			dbg('failed to create connection')
			return
		self.conn.setReuseAddr(True)
		self.creader.addConnection(self.conn)
		taskMgr.add(self.rx,"rx")
		self.rx_handle = self.rx_handle_sess
		self.tx_sess()
		self.run()

	def rx(self, data):
		if self.creader.dataAvailable():
			datagram = NetDatagram()
			if self.creader.getData(datagram):
				self.rx_handle(PyDatagramIterator.PyDatagramIterator(datagram))
		return Task.cont

	def rx_handle_sess(self, data):
		msg_type = data.getUint8()
		if msg_type != 0:
			dbg('wrong packet type: '+str(msg_type))
			return
		error = data.getUint8()
		if error == 0:
			self.rx_handle = self.rx_handle_gaming
		else:
			if error in sess_errors:
				error = sess_errors[error]
			else:
				error = str(error)+' (unknown)'
			dbg('session error '+error)

	def rx_handle_gaming(self, data):
		msg_type = data.getUint8()
		if msg_type not in self.msg_types:
			dbg('UNKNOWN PACKET TYPE '+str(msg_type))
			return
		dbg(self.msg_types[msg_type][0])
		self.msg_types[msg_type][1](data)

	def rx_sess(self, data):
		pass

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
				dbg('seq='+str(seq)+' rel=UNKNOWN ('+str(rel_type)+') len='+str(rel_len))
				data.skipBytes(rel_len)
			else:
				dbg('seq='+str(seq)+' rel='+self.rel_types[rel_type][0]+' len='+str(rel_len))
				rel = data.extractBytes(rel_len)
				dg = Datagram(rel)
				pdi = PyDatagramIterator.PyDatagramIterator(dg)
				self.rel_types[rel_type][1](pdi)
			seq = seq+1
		self.tx_ask(seq)

	def rx_rel_newwdg(self, data):
		pass

	def rx_rel_wdgmsg(self, data):
		pass

	def rx_rel_dstwdg(self, data):
		pass

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
			dbg('   id={0} name={1} version={2}'.format(tile_id,tile_name,tile_ver))

	def rx_rel_buff(self, data):
		pass

	def rx_ack(self, data):
		pass

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
		data.addZString(self.user_name)
		data.appendData(self.cookie)
		self.cwriter.send(data, self.conn, self.addr)

	def tx_ask(self, seq):
		data = PyDatagram.PyDatagram()
		data.addUint8(2) # ACK
		data.addUint16(seq)
		self.cwriter.send(data, self.conn, self.addr)

###########################################################################

hnh = hnh_client('moltke.seatribe.se', 1871, 1870)
while not hnh.authorize(u'lemings', u'lemings'):
	dbg('authorization failed')
	#TODO add delay
dbg('authorized')
hnh.start()
