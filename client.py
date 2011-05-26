#!/usr/bin/python3

import socket, ssl, hashlib, time, threading

#####################################################################

def show_bytes(s, msg):
	print(s, end='')
	for b in msg:
		print('{0:02X} '.format(b),end='')
	print('')

############################################################################

class message:
	def __init__(self, b):
		self.type = b[0]
		self.len = b[1]
		self.body = b[2:]
	def recv(self, s):
		self.type = s.recv(1)
		self.len = s.recv(1)
		self.body = s.recv(self.len)
		return self
	def send(self, s):
		return self
	def deliver(self, s):
		s.send(self.type)
		s.send(self.len)
		s.send(self.body)
		return self

############################################################################

class hnhc:
	def __init__(self, host, ssl_port, udp_port):
		self.host = host
		self.ssl_port = ssl_port
		self.udp_port = udp_port
	def authorize(self, name, password):
		s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
		ss = ssl.wrap_socket(s)
		ss.connect((self.host, self.ssl_port))

		# message(bytes(bytearray([1,len(name)])+name.encode('utf8'))).deliver(ss)
		msg = bytes(bytearray([1,len(name)])+name.encode('utf8'))
		ss.write(msg)
		# ss.send(msg)
		msg = ss.read()
		if(msg[0] != 0):
			print('username binding: wrong message type '+msg)
			#TODO: self.error = 'username response error '+error
			ss.close()
			return False

		hash = hashlib.sha256()
		hash.update(password.encode('utf8'))
		hash = hash.digest()
		msg = bytes(bytearray([2,len(hash)])+hash)
		ss.write(msg)
		msg = ss.read()
		ss.close()
		if(msg[0] != 0):
			print('password binding: wrong message type '+msg)
			return False

		self.cookie = msg[2:]
		self.name = name
		return True
	def connect(self, tries):
		self.tries = tries
		self.s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
		self.s.settimeout(4.5)
		self.s.connect((self.host, self.udp_port))

		for i in range(1,self.tries):
			msg = bytes(bytes([0,1,0])+'Haven'.encode('utf8')+bytes([0,1,0])+self.name.encode('utf8')+bytes([0])+self.cookie)
			self.s.send(msg)
			try:
				msg = self.s.recv(65535)
			except socket.timeout:
				print('timeout')
			except:
				print("unexpected recv error:", sys.exc_info()[0])
				return False
			else:
				print(msg)
				if msg[:2] == bytes([0,0]):
					self.rx = threading.Thread(target=self.receiver, name='receiver')
					self.rx.start()
					#self.rx.join()
					return True
		return False
	def receiver(self):
		while True:
			try:
				msg = self.s.recv(65535)
			except socket.timeout:
				self.s.send(bytes([3]))
				print('timeout... BEAT sent')
			except:
				print('unexpected error')
				return
			# print(msg)
			t = msg[0]
			if t == 0: # MSG_SESS
				print('SES')
				continue
			elif t == 1: # MSG_REL
				print('REL')
				seq = int(msg[1]) + (int(msg[2])<<8)
				msg = bytes([2])+msg[1:3]
				self.s.send(msg)
				print('  ack: '+str(msg))
				#reltype = msg[3]
				#if reltype & 0x80 != 0:
				#	reltype &= 0x7f
				#	rellen = int(msg[4]) + (int(msg[5])<<8)
				#	msg = msg[5:]
				#else:
				#	rellen = 'do not care'
				#print("REL: seq={0} type={1} len={2} buf={3}".format(seq,reltype,rellen,hh.s.recv(rellen)))
				#if reltype == 11: # tiles
				#	pass
			else:
				print('OTHER: '+str(msg[:5]))

	def disconnect(self):
		self.s.close()
	#def chose_character(self):

###########################################################################

hh = hnhc('moltke.seatribe.se', 1871, 1870)
if not hh.authorize('lemings', 'lemings'):
	print('authorization failed')
	exit(1)
print('authorized')
if not hh.connect(5):
	print('connection failed')
	exit(1)
print('connected')
hh.rx.join()
#hh.chose_character()
