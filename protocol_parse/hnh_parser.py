#!/usr/bin/env python
# -*- coding: utf-8 -*-

import pcapy, struct

# counters = {'tcp':0,'udp':0,'other':0}
msg = ('SESS','REL','ACK','BEAT','MAPREQ','MAPDATA','OBJDATA','OBJACK','CLOSE')
sesserr = ('OK','AUTH','BUSY','CONN','PVER','EXPR')
    # public static final int RMSG_NEWWDG = 0;
    # public static final int RMSG_WDGMSG = 1;
    # public static final int RMSG_DSTWDG = 2;
    # public static final int RMSG_MAPIV = 3;
    # public static final int RMSG_GLOBLOB = 4;
    # public static final int RMSG_PAGINAE = 5;
    # public static final int RMSG_RESID = 6;
    # public static final int RMSG_PARTY = 7;
    # public static final int RMSG_SFX = 8;
    # public static final int RMSG_CATTR = 9;
    # public static final int RMSG_MUSIC = 10;
    # public static final int RMSG_TILES = 11;
    # public static final int RMSG_BUFF = 12;
rel = ('NEWWDG','WDGMSG','DSTWDG','MAPIV','GLOBLOB','PAGINAE','RESID','PARTY','SFX','CATTR','MUSIC','TILES','BUFF')

class hnh_pkt:
	def cut(self,fmt):
		res = struct.unpack(fmt,self.data[:struct.calcsize(fmt)])
		self.data = self.data[struct.calcsize(fmt):]
		return res
	def u8(self):
		return self.cut('<B')[0]
	def u16(self):
		return self.cut('<H')[0]
	def str(self):
		tmp = self.data.index(b'\x00')
		str = self.data[:tmp].decode()
		self.data = self.data[tmp+1:]
		return str
	def b(self):
		# print(self.data)
		return b'...'
	def bytes(self,count):
		if count > 0:
			ret = self.data[:count]
			self.data = self.data[count:]
		else:
			ret = self.data[:]
			self.data = bytes()
		return ret
	def __init__(self,data,server):
		if server:
			self.desc = '   '
		else:
			self.desc = ''
		self.data = data
		self.type = self.u8()
		if self.type > 8:
			self.desc += 'UNKNOWN PACKET TYPE '+str(self.type)
			return
		self.desc += msg[self.type]+': '
		if self.type == 0: # sess
			if server:
				self.error = self.u8()
				self.desc += 'error={0}({1})'.format(self.error,sesserr[self.error])
			else:
				self.u16()
				self.proto = self.str()
				self.ver = self.u16()
				self.user = self.str()
				self.cookie = self.b()
				self.desc += 'proto={0} ver={1} user={2} cookie={3}'.format(self.proto,self.ver,self.user,self.cookie)
		elif self.type == 1: # rel
			self.seq = self.u16()
			while len(self.data) > 0:
				self.rel_type = self.u8()
				if self.rel_type&0x80 != 0:
					self.rel_type &= 0x7f;
					self.rel_len = self.u16();
				else:
					self.rel_len = 0
				self.rel = self.bytes(self.rel_len)
				self.desc += 'seq={0} type={1}({2}) len={3} rel={4}\r\n        '.format(self.seq,self.rel_type,rel[self.rel_type],self.rel_len,self.rel[:10])
				self.seq += 1
		elif self.type == 2: # ack
			self.seq = self.u16()
			self.desc += str(self.seq)
		elif self.type == 3: # beat
			pass
		elif self.type == 4: # mapreq
			pass
		elif self.type == 5: # mapdata
			pass
		elif self.type == 6: # objdata
			pass
		elif self.type == 7: # objack
			pass
		elif self.type == 8: # close
			pass
		# if len(self.data) > 0:
			# self.desc += ' remains='+str(self.data)

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
	# print('{0}.{1}.{2}.{3}:{4} -> {5}.{6}.{7}.{8}:{9}'.format(ipsrc[0],ipsrc[1],ipsrc[2],ipsrc[3],portsrc,ipdst[0],ipdst[1],ipdst[2],ipdst[3],portdst))
	if ipdst == bytes([178,63,100,209]):
		p = hnh_pkt(data,False)
	else:
		p = hnh_pkt(data,True)
	print(p.desc)

rdr = pcapy.open_offline('hh.pcap')
rdr.dispatch(40,show_info)
# print(counters)