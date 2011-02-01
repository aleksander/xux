#!/usr/bin/python3

import socket, ssl, hashlib, time

#####################################################################

def showBytes(s, msg):
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
		return self # this is cool because we can do chains like message(..).send().recv()
	def send(self, s):
		return self # this is cool because we can do chains like message(..).send().recv()
	def deliver(self, s):
		if isinstance(s, socket.socket):
			print('send over tcp')
			return self
		elif isinstance(s, ssl.SSLSocket):
			print('send over ssl')
			return self
		print('cant send over '+type(s))
		return None # this is cool because we can do chains like message(..).send().recv()

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
			print('username response error. msg: '+msg)
			#TODO: self.error = 'username response error '+error
			#ss.close()
			return False
		# replace this with:
		# message(bytes(...name...)).deliver(ss)
		hash = hashlib.sha256()
		hash.update(password.encode('utf8'))
		hash = hash.digest()
		msg = bytes(bytearray([2,len(hash)])+hash)
		ss.write(msg)
		msg = ss.read()
		ss.close()
		if(msg[0] != 0):
			print("password response error. msg: "+msg)
			return False
		# replace this with:
		# message(bytes(...digest...)).deliver(ss)
		showBytes('arecv: ', msg)
		self.cookie = msg[2:]
		showBytes('cookies:     ', self.cookie)
		self.name = name
		return True
	def connect(self, tries):
		self.tries = tries
		self.s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
		self.s.settimeout(1000)
		self.s.connect((self.host, self.udp_port))

		msg = bytes(bytearray([0,1,0])+'Haven'.encode('utf8')+bytearray([0,1,0])+self.name.encode('utf8')+bytearray([0])+self.cookie)
		showBytes('send: ', msg)
		self.s.send(msg)
		msg = self.s.recv(65535)
		#if !recv:
		#if try>tries return False
		#wait 2 secs
		#try send SESS again
		showBytes('recv: ', msg)
	def disconnect(self):
		self.s.close()
	#def chose_character(self):

###########################################################################

hh = hnhc('moltke.seatribe.se', 1871, 1870)
if hh.authorize('lemings', 'lemings'):
	if hh.connect(5):
		pass
		#hh.chose_character()
