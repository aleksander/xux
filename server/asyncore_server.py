#!/usr/bin/env python3.2
# -*- coding: utf-8 -*-

import asyncore
import socket
import ssl

class AuthHandler(asyncore.dispatcher_with_send):
	def __init__(self, conn):
        asyncore.dispatcher_with_send.__init__(self, conn)
        self.socket = ssl.wrap_socket(conn, server_side=True, certfile='keycert.pem', do_handshake_on_connect=False)
        while True:
            try:
                self.socket.do_handshake()
                break
            except ssl.SSLError, err:
                if err.args[0] == ssl.SSL_ERROR_WANT_READ:
                    select.select([self.socket], [], [])
                elif err.args[0] == ssl.SSL_ERROR_WANT_WRITE:
                    select.select([], [self.socket], [])
                else:
                    raise
		
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
		handler = AuthHandler(sock)

	def handle_error(self):
		raise

auth_server = AuthServer('', 1871)
asyncore.loop()