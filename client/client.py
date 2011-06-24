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
	sess_errors = {0:'OK', 1:'AUTH', 2:'BUSY', 3:'CONN', 4:'PVER', 5:'EXPR'}
	self.msg_types = {
		0:('SESS',self.rx_sess),
		1:('REL',self.rx_rel),
		2:('ACK',self.rx_ack),
		3:('BEAT',self.rx_beat),
		4:('MAPREQ',self.rx_mapreq),
		5:('MAPDATA',self.rx_mapdata),
		6:('OBJDATA',self.rx_objdata),
		7:('OBJACK',self.rx_objack),
		8:('CLOSE'self.rx_close)
	}
	rel_types = {0:'NEWWDG',1:'WDGMSG',2:'DSTWDG',3:'MAPIV',4:'GLOBLOB',5:'PAGINAE',6:'RESID',
                 7:'PARTY',8:'SFX',9:'CATTR',10:'MUSIC',11:'TILES',12:'BUFF'}
	def __init__(self, host, ssl_port, udp_port):
		self.host = host
		self.ssl_port = ssl_port
		self.udp_port = udp_port
		self.addr = NetAddress()
		self.addr.setHost(self.host, self.udp_port)
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
		self.sess()
		self.run()
	def sess(self):
		data = PyDatagram.PyDatagram()
		data.addUint8(0) # SESS
		data.addUint16(1) # ???
		data.addZString(u'Haven') # protocol name
		data.addUint16(2) # version
		data.addZString(self.user_name)
		data.appendData(self.cookie)
		self.cwriter.send(data, self.conn, self.addr)
	def ask(self, seq):
		data = PyDatagram.PyDatagram()
		data.addUint8(2) # ACK
		data.addUint16(seq)
		self.cwriter.send(data, self.conn, self.addr)
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
		if msg_type not in msg_types:
			dbg('UNKNOWN PACKET TYPE {}'.format(msg_type))
			return
		dbg(str(msg_type)+' ('+msg_types[msg_type]+')')
		# TODO: replace with msg_types[msg_type][handler](data)
		if msg_type == 0: # 'SESS'
			pass
		elif msg_type == 1: # 'REL'
			seq = data.getUint16()
			while data.getRemainingSize():
				rel_type = data.getUint8()
				if rel_type&0x80 != 0:
					rel_type &= 0x7f;
					rel_len = data.getUint16()
				else:
					rel_len = data.getRemainingSize()
				# TODO: replace with rel_types[rel_type][handler](data)
				if rel_type == 0: # 'NEWWDG'
					pass
				elif rel_type == 1: # 'WDGMSG'
					pass
				elif rel_type == 2: # 'DSTWDG'
					pass
				elif rel_type == 3: # 'MAPIV'
					pass
				elif rel_type == 4: # 'GLOBLOB'
					pass
				elif rel_type == 5: # 'PAGINAE'
					pass
				elif rel_type == 6: # 'RESID'
					pass
				elif rel_type == 7: # 'PARTY'
					pass
				elif rel_type == 8: # 'SFX'
					pass
				elif rel_type == 9: # 'CATTR'
					pass
				elif rel_type == 10: # 'MUSIC'
					pass
				elif rel_type == 11: # 'TILES'
					pass
				elif rel_type == 12: # 'BUFF'
					pass
				data.skipBytes(rel_len)
				# TODO: if rel_type not in rel_types: ...
				dbg('seq='+str(seq)+' rel='+rel_types[rel_type]+' len='+str(rel_len))
				seq = seq+1
			self.ask(seq)
		elif msg_type == 2: # 'ACK'
			pass
		elif msg_type == 3: # 'BEAT'
			pass
		elif msg_type == 4: # 'MAPREQ'
			pass
		elif msg_type == 5: # 'MAPDATA'
			pass
		elif msg_type == 6: # 'OBJDATA'
			pass
		elif msg_type == 7: # 'OBJACK'
			pass
		elif msg_type == 8: # 'CLOSE'
			pass

###########################################################################

hnh = hnh_client('moltke.seatribe.se', 1871, 1870)
while not hnh.authorize(u'lemings', u'lemings'):
	dbg('authorization failed')
	#TODO add delay
dbg('authorized')
hnh.start()
