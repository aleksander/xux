#!/usr/bin/env python
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
			ctx.load_cert_chain(certfile="authsrv.pem")
		except:
			print("error: {} {}".format(sys.exc_info()[1],sys.exc_info()[2]))
			return
		s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
		s.bind(('', self.ssl_port))
		s.listen(5)
		while True:
			newsocket, fromaddr = s.accept()
			print('connected: ', fromaddr)
			ss = ctx.wrap_socket(newsocket, server_side=True)
			try:
				while True:
					data = ss.recv(1024)
					if not data:
						break
					print(data)
			finally:
				connstream.shutdown(socket.SHUT_RDWR)
				connstream.close()
		
		# s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
		# ss = ssl.wrap_socket(sock=s, server_side=True)
		# ss.listen(1)
		# conn, addr = ss.accept()
		# print('Connected: ', addr)
		# while True:
			# data = conn.recv(1024)
			# if not data: break
			# print(data)
			# conn.send(data)
		# conn.close()

srv = hnh_server(1871, 1870)
srv.start()