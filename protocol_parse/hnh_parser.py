#!/usr/bin/env python
# -*- coding: utf-8 -*-

import pcapy

counters = {'tcp':0,'udp':0,'other':0}

def show_info(hdr,data):
	# print(str(hdr.getcaplen())+' ('+str(hdr.getlen())+')')
	# struct.unpack('', data)
	# print('{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} <- {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}'.format(*data[:12]))
	if data[12:14] == b'\x08\x00':
		data = data[14:]
		if data[9:10] == b'\x11':
			counters['udp'] += 1
			data = data[(data[0]&0xf)*4:]
			print('{0:d} -> {1:d}'.format(data[:2],data[2:4]))
		elif data[9:10] == b'\x06':
			counters['tcp'] += 1
		else:
			counters['other'] += 1
			print(data[9:10])

rdr = pcapy.open_offline('hh.pcap')
rdr.dispatch(-1,show_info)
print(counters)