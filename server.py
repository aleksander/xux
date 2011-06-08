#!/usr/bin/python3.2
##﻿!/usr/bin/env python3.2
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
		try:
			ctx.load_cert_chain(certfile="crt.pem",keyfile="key.pem")
		except:
			print("error: {} {}".format(sys.exc_info()[1],sys.exc_info()[2]))
			return
		s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
		s.bind(('', self.ssl_port))
		s.listen(5)
		#while True:
		
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
		s.listen(5)
		data = ss.recv(1024)
		

srv = hnh_server(1871, 1870)
srv.start()
