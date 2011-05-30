#!/usr/bin/env python
# -*- coding: utf-8 -*-

import pcapy, struct

counters = {'tcp':0,'udp':0,'other':0}

    # public static final int MSG_SESS = 0;
    # public static final int MSG_REL = 1;
    # public static final int MSG_ACK = 2;
    # public static final int MSG_BEAT = 3;
    # public static final int MSG_MAPREQ = 4;
    # public static final int MSG_MAPDATA = 5;
    # public static final int MSG_OBJDATA = 6;
    # public static final int MSG_OBJACK = 7;
    # public static final int MSG_CLOSE = 8;

msg = ('sess','rel','ack','beat','mapreq','mapdata','objdata','objack','close')

def hnh_protocol_parse(data):
	print('   '+msg[data[0]]+': '+str(data[1:25]))
def show_info(hdr,data):
	fmt = '!6s6sH'
	(macdst,macsrc,ethertype) = struct.unpack(fmt,data[:struct.calcsize(fmt)])
	if ethertype != 0x0800:
		print('not IP !!!')
		return
	data = data[struct.calcsize(fmt):]
	fmt = '!BBHHHBBH4s4s'
	(vhl,dscp,len,id,ffo,ttl,proto,crc,ipsrc,ipdst) = struct.unpack(fmt,data[:struct.calcsize(fmt)])
	if vhl&0xf > 5:
		print('ip options !!!')
		return
	if proto != 0x11:
		# print('not UDP !!!')
		return
	data = data[struct.calcsize(fmt):]
	fmt = '!HHHH'
	(portsrc,portdst,len,crc) = struct.unpack(fmt,data[:struct.calcsize(fmt)])
	data = data[struct.calcsize(fmt):]
	print('{0}.{1}.{2}.{3}:{4} -> {5}.{6}.{7}.{8}:{9}'.format(ipsrc[0],ipsrc[1],ipsrc[2],ipsrc[3],portsrc,ipdst[0],ipdst[1],ipdst[2],ipdst[3],portdst))
	hnh_protocol_parse(data)

rdr = pcapy.open_offline('hh.pcap')
rdr.dispatch(-1,show_info)
print(counters)