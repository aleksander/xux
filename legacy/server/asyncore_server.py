#!/usr/bin/env python3.2
# -*- coding: utf-8 -*-

import asyncore
import select
import socket
import ssl

class AuthHandler(asyncore.dispatcher_with_send):
	def __init__(self, sock, ctx):
		asyncore.dispatcher_with_send.__init__(self, sock)
		self.sock = ctx.wrap_socket(sock, server_side=True, do_handshake_on_connect=False)
		while True:
			try:
				self.sock.do_handshake()
				break
			except ssl.SSLError as err:
				if err.args[0] == ssl.SSL_ERROR_WANT_READ:
					select.select([self.sock], [], [])
				elif err.args[0] == ssl.SSL_ERROR_WANT_WRITE:
					select.select([], [self.sock], [])
				else:
					raise
		self.state = 0

	def readable(self):
		if isinstance(self.sock, ssl.SSLSocket):
			while self.sock.pending() > 0:
				self.handle_read_event()
		return True

	def handle_read(self):
		try:
			data = self.sock.recv(1024)
			# if not data:
				# # TODO throw exception
				# pass
			print(data)
			if self.state == 0:
				self.sock.send(b'\x00\x00')
				self.state = 1
			elif self.state == 1:
				self.sock.send(b'\x00\x08deadbeef')
				print('auth passed')
				self.state = 2
			# else:
				# self.sock.shutdown(socket.SHUT_RDWR)
				# self.sock.close()
		except:
			print('connection lost')
			self.sock.shutdown(socket.SHUT_RDWR)
			self.sock.close()


	def handle_error(self):
		raise

class AuthServer(asyncore.dispatcher):
	def __init__(self, host, port):
		asyncore.dispatcher.__init__(self)
		self.ctx = ssl.SSLContext(ssl.PROTOCOL_TLSv1)
		self.ctx = ssl.SSLContext(ssl.PROTOCOL_SSLv23)
		self.ctx.options |= ssl.OP_NO_SSLv2
		self.ctx.load_cert_chain(certfile='crt.pem', keyfile='key.pem')
		self.create_socket(socket.AF_INET, socket.SOCK_STREAM)
		self.set_reuse_addr()
		self.bind((host, port))
		self.listen(5)
		print('started')

	def handle_accepted(self, sock, addr):
		print('accepted: ',repr(addr))
		handler = AuthHandler(sock, self.ctx)

	def handle_error(self):
		raise

auth_server = AuthServer('', 1871)
asyncore.loop()