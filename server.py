#!/usr/bin/python3.2
##!/usr/bin/env python3.2
# -*- coding: utf-8 -*-

import socket, ssl, sys

class hnh_server:
	def __init__(self, ssl_port, udp_port):
		self.ssl_port = ssl_port
		self.udp_port = udp_port
	def start(self):
		ctx = ssl.SSLContext(ssl.PROTOCOL_TLSv1)
		ctx = ssl.SSLContext(ssl.PROTOCOL_SSLv23)
		ctx.options |= ssl.OP_NO_SSLv2
		ctx.load_cert_chain(certfile='crt.pem', keyfile='key.pem')
		s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
		s.bind(('', self.ssl_port))
		s.listen(5)
		
		newsocket, fromaddr = s.accept()
		print('connected: ', fromaddr)
		ss = ctx.wrap_socket(newsocket, server_side=True)
		try:
			data = ss.recv(1024)
			if not data:
				#TODO throw exception
				pass
			print(data)
			ss.send(b'\x00\x00')
			data = ss.recv(1024)
			if not data:
				#TODO throw exception
				pass
			print(data)
			ss.send(b'\x00\x08deadbeef')
			print('auth passed')
		finally:
			ss.shutdown(socket.SHUT_RDWR)
			ss.close()
			
		s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
		s.bind(('', self.udp_port))

		data, rs = s.recvfrom(1024)
		s.connect(rs)
		print('connected: ',rs)
		print(data)
		s.send(b'\x00\x00')
		while True:
			data = s.recv(1024)
			print(data)
			if data[0] == 3:
				break
		s.send(b'\x01\x00\x00\x00\x01\x00cnt\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x03\x20\x03\x00\x00\x03\x58\x02\x00\x00')
		#s.recv(1024)
		print(data)
		s.send(b'\x01\x01\x00\x00\x02\x00charlist\x00\x0f\x00\x00\x00\x0f\x00\x00\x00\x01\x00\x01\x03\x00\x00\x00')
		try:
			while True:
				data = s.recv(1024)
				print(data)
		except:
			print('interrupted')
			s.send(b'\x08')
			s.send(b'\x08')
			s.send(b'\x08')
			s.close()

srv = hnh_server(1871, 1870)
srv.start()
