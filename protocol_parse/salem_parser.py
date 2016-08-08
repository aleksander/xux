﻿#!/usr/bin/python
# -*- coding: utf-8 -*-


import pure_pcapy as pcapy
import struct, sys, zlib, math
from sys import argv


class Struct:
        def __init__(self, **kwds):
                self.__dict__.update(kwds)

wdg_list_types = {
	0:'END',
	1:'INT',
	2:'STR',
	3:'COORD',
	4:'UINT8',
	5:'UINT16',
	6:'COLOR',
	8:'TTOL',
	9:'INT8',
	10:'INT16',
	12:'NIL',
	14:'BYTES',
	15:'FLOAT32',
	16:'FLOAT64'}

class Message:
	def __init__(self, data):
		self.data = bytearray(data)

	def unpack (self, fmt):
		size = struct.calcsize(fmt)
		(ret,) = struct.unpack(fmt,self.data[:size])
		self.data[0:size] = []
		return ret

	@property
	def s8 (self):
		return self.unpack('b')

	@property
	def u8 (self):
		return self.unpack('B')

	@property
	def s16 (self):
		return self.unpack('<h')

	@property
	def u16 (self):
		return self.unpack('<H')

	@property
	def cstr (self):
		i = self.data.index(b'\x00')
		str = self.data[:i].decode()
		self.data[0:i+1] = []
		return str

	def b (self, count=0):
		if count > 0:
			ret = self.data[:count]
			self.data[0:count] = []
		else:
			ret = self.data[:]
			self.data[:] = []
		return ret

	@property
	def s32 (self):
		return self.unpack('<i')

	@property
	def u32 (self):
		return self.unpack('<I')

	@property
	def f32 (self):
		return self.unpack('<f')

	@property
	def f64 (self):
		return self.unpack('<d')

	@property
	def len(self):
		return len(self.data)

	@property
	def list(self):
		l = []
		while self.len > 0:
			wdg_lt = self.u8
			if wdg_lt not in wdg_list_types:
				raise Exception('UNKNOWN LIST TYPE {}'.format(wdg_lt))
			if wdg_lt == 0: # END
				break
			elif wdg_lt == 1: # INT
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.s32))
			elif wdg_lt == 2: # STR
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.cstr))
			elif wdg_lt == 3: # COORD
				l.append(Struct(type=wdg_list_types[wdg_lt], value=[self.s32,self.s32]))
			elif wdg_lt == 4: # UINT8
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.u8))
			elif wdg_lt == 5: # UINT16
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.u16))
			elif wdg_lt == 6: # COLOR
				l.append(Struct(type=wdg_list_types[wdg_lt], value=[self.u8,self.u8,self.u8,self.u8]))
			elif wdg_lt == 8: # TTOL
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.list))
			elif wdg_lt == 9: # INT8
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.s8))
			elif wdg_lt == 10: # INT16
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.s16))
			elif wdg_lt == 12: # NIL
				l.append(Struct(type=wdg_list_types[wdg_lt], value='null'))
			elif wdg_lt == 14: # BYTES
				bytes_len = self.u8
				if (bytes_len & 128) != 0:
					bytes_len = self.s32
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.b(bytes_len)))
			elif wdg_lt == 15: # FLOAT32
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.f32))
			elif wdg_lt == 16: # FLOAT64
				l.append(Struct(type=wdg_list_types[wdg_lt], value=self.f64))
		return l

	@property
	def coord(self):
		return [self.s32,self.s32]

class SalemProtocolParser:
	def __init__(self):
		self.objdata = {}
		self.resids = {}
		self.fragbufs = {}
		self.sess_errors = {
			0:'OK',
			1:'AUTH',
			2:'BUSY',
			3:'CONN',
			4:'PVER',
			5:'EXPR'
		}
		self.msg_types = {
			0:Struct(name =    'SESS', parse = self.rx_sess),
			1:Struct(name =     'REL', parse = self.rx_rel),
			2:Struct(name =     'ACK', parse = self.rx_ack),
			3:Struct(name =    'BEAT', parse = self.rx_beat),
			4:Struct(name =  'MAPREQ', parse = self.rx_mapreq),
			5:Struct(name = 'MAPDATA', parse = self.rx_mapdata),
			6:Struct(name = 'OBJDATA', parse = self.rx_objdata),
			7:Struct(name =  'OBJACK', parse = self.rx_objack),
			8:Struct(name =   'CLOSE', parse = self.rx_close)
		}
		self.rel_types = {
			0: Struct(name =  'NEWWDG', parse = self.rx_rel_newwdg),
			1: Struct(name =  'WDGMSG', parse = self.rx_rel_wdgmsg),
			2: Struct(name =  'DSTWDG', parse = self.rx_rel_dstwdg),
			3: Struct(name =   'MAPIV', parse = self.rx_rel_mapiv),
			4: Struct(name = 'GLOBLOB', parse = self.rx_rel_globlob),
			5: Struct(name = 'PAGINAE', parse = self.rx_rel_paginae),
			6: Struct(name =   'RESID', parse = self.rx_rel_resid),
			7: Struct(name =   'PARTY', parse = self.rx_rel_party),
			8: Struct(name =     'SFX', parse = self.rx_rel_sfx),
			9: Struct(name =   'CATTR', parse = self.rx_rel_cattr),
			10:Struct(name =   'MUSIC', parse = self.rx_rel_music),
			11:Struct(name =   'TILES', parse = self.rx_rel_tiles),
			12:Struct(name =    'BUFF', parse = self.rx_rel_buff),
			13:Struct(name = 'SESSKEY', parse = self.rx_rel_sesskey)
		}
		self.objdata_types = {
			0:  Struct(name =     'OD_REM', parse = self.rx_objdata_rem),
			1:  Struct(name =    'OD_MOVE', parse = self.rx_objdata_move),
			2:  Struct(name =     'OD_RES', parse = self.rx_objdata_res),
			3:  Struct(name =  'OD_LINBEG', parse = self.rx_objdata_linbeg),
			4:  Struct(name = 'OD_LINSTEP', parse = self.rx_objdata_linstep),
			5:  Struct(name =  'OD_SPEECH', parse = self.rx_objdata_speech),
			6:  Struct(name = 'OD_COMPOSE', parse = self.rx_objdata_compose),
			7:  Struct(name = 'OD_DRAWOFF', parse = self.rx_objdata_drawoff),
			8:  Struct(name =   'OD_LUMIN', parse = self.rx_objdata_lumin),
			9:  Struct(name =  'OD_AVATAR', parse = self.rx_objdata_avatar),
			10: Struct(name =  'OD_FOLLOW', parse = self.rx_objdata_follow),
			11: Struct(name =  'OD_HOMING', parse = self.rx_objdata_homing),
			12: Struct(name = 'OD_OVERLAY', parse = self.rx_objdata_overlay),
			13: Struct(name =    'OD_AUTH', parse = self.rx_objdata_auth),
			14: Struct(name =  'OD_HEALTH', parse = self.rx_objdata_health),
			15: Struct(name =   'OD_BUDDY', parse = self.rx_objdata_buddy),
			16: Struct(name = 'OD_CMPPOSE', parse = self.rx_objdata_cmppose),
			17: Struct(name =  'OD_CMPMOD', parse = self.rx_objdata_cmpmod),
			18: Struct(name =  'OD_CMPEQU', parse = self.rx_objdata_cmpequ),
			19: Struct(name =    'OD_ICON', parse = self.rx_objdata_icon),
			255:Struct(name =     'OD_END', parse = self.rx_objdata_end)}

	def parse (self, _data, server):
		if server:
			print('SERVER')
		else:
			print('CLIENT')
		data = Message(_data)
		type = data.u8
		if type not in self.msg_types:
			print(' UNKNOWN PACKET TYPE {}'.format(type))
			return
		print(' {}'.format(self.msg_types[type].name),end='')
		self.msg_types[type].parse(data,server)
		if data.len > 0:
			print('data remains: {}'.format(data.data))
		
	######## SESS #################################
	def rx_sess(self, data, server):
		print()
		if server:
			error = data.u8
			print('  error={}({})'.format(error,self.sess_errors[error]))
		else:
			unknown = data.u16 # ???
			proto = data.cstr
			ver = data.u16
			user = data.cstr
			cookie = data.b()
			print('  unknown={} proto={} ver={} user={} cookie={}'.format(unknown, proto,ver,user,cookie))

	######## REL ##################################
	def rx_rel (self, data, server): # Session.java +488
		seq = data.u16
		print('  seq={0}'.format(seq))
		while data.len > 0:
			rel_type = data.u8
			if rel_type&0x80 != 0:
				rel_type &= 0x7f
				rel_len = data.u16
			else:
				rel_len = data.len
			rel = Message(data.b(rel_len))
			if rel_type not in self.rel_types:
				print('  UNKNOWN ({}) len={}'.format(rel_type, rel_len))
				return
			else:
				print('  {} len={}'.format(self.rel_types[rel_type].name, rel_len))
				self.rel_types[rel_type].parse(rel)
			if rel.len > 0:
				print('rel remains: {}'.format(rel.data))

	def rx_rel_newwdg (self, data):
		wdg_id = data.u16
		wdg_type = data.cstr
		wdg_parent = data.u16
		pargs = data.list
		cargs = data.list
		print('   id={} type={} parent={}'.format(wdg_id,wdg_type,wdg_parent))
		print('    pargs:')
		for elem in pargs:
			print('     {} : {}'.format(elem.type,elem.value))
		print('    cargs:')
		for elem in cargs:
			print('     {} : {}'.format(elem.type,elem.value))

	def print_list (self, list, indent):
		for elem in list:
			if elem.type == 'TTOL':
				print('{}{} : ['.format(indent,elem.type))
				self.print_list(elem.value,indent+'  ');
				print('{}]'.format(indent))
			else:
				print('{}{} : {}'.format(indent,elem.type,elem.value))
		

	def rx_rel_wdgmsg (self, data):
		wdg_id = data.u16
		wdg_msg_name = data.cstr
		wdg_msg = data.list
		print('   id={} name={}'.format(wdg_id,wdg_msg_name))
		self.print_list(wdg_msg,'     ')

	def rx_rel_dstwdg (self, data): #destroy widget
		id = data.u16
		print('   id={}'.format(id))

	def rx_rel_mapiv (self, data):
		mapiv_type = data.u8
		if mapiv_type == 0: # ???
			print('    invalidate coord={}'.format([data.s32,data.s32]))
		elif mapiv_type == 1: # ???
			print('    trim ul={} lr={}'.format([data.s32,data.s32],[data.s32,data.s32]))
		elif mapiv_type == 2: # ???
			print('    trim all')

	def rx_rel_globlob (self, data): # Glob.java +217
		gmsg_types = {0:'TIME',2:'LIGHT',3:'SKY'}
		inc = data.u8 != 0
		return
		while data.len > 0:
			gmsg_type = data.u8
			if gmsg_type not in gmsg_types:
				raise Exception('UNKNOWN GMSG TYPE {}'.format(gmsg_type))
			print('    {} '.format(gmsg_types[gmsg_type]),end='')
			if gmsg_type == 0: # TIME
				print(data.s32)
			elif gmsg_type == 2: # LIGHT
				ambient = [data.u8,data.u8,data.u8,data.u8]
				diffuse = [data.u8,data.u8,data.u8,data.u8]
				specular = [data.u8,data.u8,data.u8,data.u8]
				angle = data.s32 / 1000000.0 * math.pi * 2.0
				elev = data.s32 / 1000000.0 * math.pi * 2.0
				print('amb={} diff={} spec={} ang={} elev={}'.format(ambient,diffuse,specular,angle,elev))
			elif gmsg_type == 3: # SKY
				id1 = data.u16
				if id1 == 65535:
					print('sky1=null sky2=null skyblend=0.0')
				else:
					id2 = data.u16
					if id2 == 65535:
						print('sky1=getres({}) sky2=null skyblend=null'.format(id1))
					else:
						skyblend = data.s32 / 1000000.0
						print('sky1=getres({}) sky2=getres({}) skyblend={}'.format(id1,id2,skyblend))

	def rx_rel_paginae (self, data): # Glob.java +293
		while data.len > 0:
			act = data.u8
			if act == int(b'+'[0]):
				nm = data.cstr
				ver = data.u16
				tmp = ''
				while True:
					t = data.u8
					if t == 0:
						break
					elif t == int(b'!'[0]):
						tmp += ' (!)'
					elif t == int(b'*'[0]):
						meter = data.s32
						dtime = data.s32
						tmp += ' (*) meter={} dtime={}'.format(meter,dtime)
					elif t == int(b'^'[0]):
						tmp += ' (^)'
				print('    act={}(+) nm={} ver={} {}'.format(act,nm,ver,tmp))
			elif act == int(b'-'[0]):
				nm = data.cstr
				ver = data.u16
				print('    act={}(-) nm={} ver={}'.format(act,nm,ver))
			else:
				raise Exception('unknow pagina action')

	def rx_rel_resid (self, data):
		res_id = data.u16
		res_name = data.cstr
		res_ver = data.u16
		print('   id={} name={} ver={}'.format(res_id,res_name,res_ver))
		self.resids[res_id] = Struct(name=res_name,ver=res_ver)

	def rx_rel_party (self, data):
		while data.len > 0:
			party_type = data.u8
			if party_type == 0: # LIST
				print('   LIST')
				while True:
					party_id = data.s32
					if party_id < 0:
						break
					print('    id={}'.format(party_id))
			elif party_type == 1: # LEADER
				print('   LEADER id={}'.format(data.s32))
			elif party_type == 2: # MEMBER
				print('   MEMBER id={} vis={} coord={} color={}'
				.format(data.s32,data.u8,[data.s32,data.s32],[data.u8,data.u8,data.u8,data.u8]))

	def rx_rel_sfx (self, data):
		print('   res={} vol={} spd={}'.format(data.u16,data.u16,data.u16))

	def rx_rel_cattr (self, data):
		while data.len > 0:
			attr_name = data.cstr
			attr_base = data.s32
			attr_comp = data.s32
			print('   name={} base={} comp={}'.format(attr_name,attr_base,attr_comp))

	def rx_rel_music (self, data):
		music_name = data.cstr
		music_ver = data.u16
		if data.len > 0:
			music_loop = data.u8
		else:
			music_loop = 0
		print('   name={} ver={} loop={}'.format(music_name,music_ver,music_loop))

	def rx_rel_tiles (self, data):
		while data.len > 0:
			tile_id = data.u8
			tile_name = data.cstr
			tile_ver = data.u16
			print('   id={0} name={1} version={2}'.format(tile_id,tile_name,tile_ver))

	def rx_rel_buff (self, data):
		buff_name = data.cstr
		if buff_name == 'clear':
			print('   clear buffers')
		elif buff_name == 'set':
			print('   set buffers id={} res={} tt={} ameter={} nmeter={} cmeter={} cticks={} major={}'
			.format(data.s32,data.u16,data.cstr,data.s32,data.s32,data.s32,data.s32,data.u8))
		elif buff_name == 'rm':
			print('   remove buffers id={}'.format(data.s32))

	def rx_rel_sesskey (self, data):
		sess_key = data.b()
		print('   sess_key={}'.format(sess_key))

	######## ACK ##################################
	def rx_ack (self, data, server):
		seq = data.u16
		print('  seq={}'.format(seq))

	######## BEAT #################################
	def rx_beat (self, data, server):
		print()

	######## MAPREQ ###############################
	def rx_mapreq (self, data, server):
		print()
		print('  coord={}'.format([data.s32,data.s32]))

	######## MAPDATA ##############################
	def rx_mapdata (self, data, server):
		print()
		pktid = data.s32
		off = data.u16
		length = data.u16
		unknown = data.b(8)
		segment = data.b()
		print('   pktid={} off={} len={} !!! TODO parse fragbuf'.format(pktid,off,length))
		
		fragbufs = self.fragbufs
		fragbuf = fragbufs.get(pktid)
		if fragbuf == None:
			fragbuf = bytearray(length)
			fragbufs[pktid] = fragbuf
		fragbuf[off:off+len(segment)] = segment
		# MCache.java +444
		# Defrag.java +55

	def parse_fragbufs (self):
		for id,buf in self.fragbufs.items():
			data = Message(buf)
			coord = data.coord
			print('{} {}'.format(id,coord))
			## MCache.java +278
			#mmname = data.cstr
			#pfl = bytearray(256)
			#while True:
			#	pidx = data.u8
			#	if pidx == 255:
			#		break
			#	pfl[pidx] = data.u8
			#data = Message(zlib.decompress(data.data))
			#print('coord={} mmname={} pfl={}'.format(coord,mmname,pfl))

#coord = [data.s32,data.s32]
#mmname = data.cstr
#pfl = []
#while True:
#       pfl.append(data.u8)
#       if pfl[-1] == 255:
#               pfl[-1:] = []
#               break
#dec_data = Message(zlib.decompress(data.data))
#tiles = dec_data.b(100*100)
#pidx = dec_data.u8
#if pidx != 0xff:
#       print('  !!! FIXME') 
#print('   pktid={} off={} len={} grid_coord={} mmname="{}" pfl={}'.format(pktid,off,length,coord,mmname,pfl))
#for i in range(0,100):
#       print('   ',end='')
#       for j in range(0,100):
#               print('{:02X}'.format(tiles[i*100+j]), end='')
#       print('')



#    public void mapdata2(Message msg) {
#        Coord c = msg.coord();
#        synchronized(grids) {
#            synchronized(req) {
#                if(req.containsKey(c)) {
#                    Grid g = grids.get(c);
#                    if(g == null)
#                        grids.put(c, g = new Grid(c));
#                    g.fill(msg);
#                    req.remove(c);
#                    olseq++;
#                }
#            }
#        }
#    }
#
#
#

	######## OBJDATA ##############################
	def rx_objdata (self, data, server): # Session.java +241
		print()
		while data.len > 0:
			fl = data.u8
			id = data.s32
			frame = data.s32
			print('  id={} frame={}'.format(id,frame))
			if (id == 0):
				print('!!! SOME PACKET BREAKGE (pure_pcap bug?)')
				return
			if (fl & 1) != 0:
				print('   remove id={} frame={}'.format(id,frame-1))
			obj = Struct(fl=fl,frame=frame,coord=None,resid=None)
			while True:
				type = data.u8
				if type not in self.objdata_types:
					raise Exception('unknown objdata type {}'.format(type))
				print('   {} '.format(self.objdata_types[type].name),end=' ')
				if type == 255: # OD_END
					print('')
					break
				self.objdata_types[type].parse(data, obj)
			if id not in self.objdata:
				self.objdata[id] = obj
			else:
				_obj = self.objdata[id]
				_obj.fl = obj.fl
				_obj.frame = obj.frame
				if obj.coord != None:
					_obj.coord = obj.coord
				if obj.resid != None:
					_obj.resid = obj.resid

	def rx_objdata_rem (self, data, obj):
		print('remove')

	def rx_objdata_move (self, data, obj):
		coord = data.coord
		ia = data.u16
		print('coord={} ia={}'.format(coord, ia))
		obj.coord = coord

	def rx_objdata_res (self, data, obj):
		resid = data.u16
		sdt = None
		if (resid & 0x8000) != 0: #TODO if resid.bit(4).is_set ...
			resid &= ~0x8000
			sdt = data.b(data.u8)
		print('resid={} sdt={}'.format(resid,sdt))
		obj.resid = resid

	def rx_objdata_linbeg (self, data, obj):
		print('s={} t={} c={}'.format(data.coord,data.coord,data.s32))

	def rx_objdata_linstep (self, data, obj):
		print('l={}'.format(data.s32))

	def rx_objdata_speech (self, data, obj):
		print('zo={} text={}'.format(data.s16,data.cstr))

	def rx_objdata_compose (self, data, obj):
		print('resid={}'.format(data.u16))

	def rx_objdata_drawoff (self, data, obj):
		print('off={}'.format(data.coord))

	def rx_objdata_lumin (self, data, obj):
		print('off={} sz={} str={}'.format(data.coord,data.u16,data.u8))

	def rx_objdata_avatar (self, data, obj):
		layers = []
		while True:
			layer = data.u16
			if layer == 65535:
				break
			layers.append(layer)
		print('layers={}'.format(layers))

	def rx_objdata_follow (self, data, obj):
		oid = data.u32
		if oid != 0xffFFffFF:
			print('oid={} xfres={} xfname={}'.format(oid,data.u16,data.cstr))
		else:
			print('oid={}'.format(oid))

	def rx_objdata_homing (self, data, obj):
		oid = data.u32
		print('oid={}'.format,end=' ')
		if oid == 0xffFFffFF:
			print('homostop')
		elif oid == 0xffFFffFe:
			print('homocoord coord={} v={}'.format(data.coord,data.u16))
		else:
			print('homing coord={} v={}'.format(data.coord,data.u16))

	def rx_objdata_overlay (self, data, obj):
		olid = data.s32
		prs = (olid & 1) != 0
		olid >>= 1
		resid = data.u16
		if resid == 65535:
			resid = None
			sdt = None
		elif (resid & 0x8000) != 0:
			resid &= ~0x8000
			sdt = data.b(data.u8)
		else:
			sdt = []
		print('olid={} prs={} resid={} sdt={}'.format(olid,prs,resid,sdt))

	def rx_objdata_auth (self, data, obj):
		raise Exception('incorrect objdata type AUTH')

	def rx_objdata_health (self, data, obj):
		print('hp={}'.format(data.u8))

	def rx_objdata_buddy (self, data, obj):
		print('name={} group={} btype={}'.format(data.cstr,data.u8,data.u8))

	def rx_objdata_cmppose (self, data, obj):
		pfl = data.u8
		seq = data.u8
		print('pfl={} seq={}'.format(pfl,seq))
		if (pfl & 2) != 0:
			while True:
				resid = data.u16
				sdt = None
				if resid == 65535:
					break
				if resid & 0x8000 != 0:
					resid &= ~0x8000
					sdt = data.b(data.u8)
				print('         resid={} sdt={}'.format(resid,sdt))
		if (pfl & 4) != 0:
			while True:
				resid = data.u16
				sdt = None
				if resid == 65535:
					break
				if (resid & 0x8000) != 0:
					resid &= ~0x8000
					sdt = data.b(data.u8)
				print('         resid={} sdt={}'.format(resid,sdt))
			ttime = data.u8
			print('         ttime={}'.format(ttime))

	def rx_objdata_cmpmod (self, data, obj):
		while True:
			modif = data.u16
			if modif == 65535:
				break
			while True:
				resid = data.u16
				if resid == 65535:
					break
		print('!!! TODO print all this')

	def rx_objdata_cmpequ (self, data, obj):
		while True:
			h = data.u8
			if h == 255:
				break
			at = data.cstr
			resid = data.u16
			ef = h & 0x80
			if (ef & 128) != 0:
				x = data.s16
				y = data.s16
				z = data.s16
		print('!!! TODO print all this')

	def rx_objdata_icon (self, data, obj):
		resid = data.u16
		if resid == 65535:
			print('icon=null')
		else:
			ifl = data.u8
			print('icon=getres({}) ifl={}'.format(resid,ifl))

	def rx_objdata_end (self, data, obj):
		pass
			
	######## OBJACK ###############################
	def rx_objack (self, data, server):
		print()
		while data.len > 0:
			print('   id={} frame={}'.format(data.s32,data.s32))

	######## CLOSE ################################
	def rx_close (self, data, server):
		print()


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
	if portsrc == 1870:
		if portsrc == portdst:
			print('SOURCE PORT == DEST PORT')
			exit(1)
		if parse_server_packets:
			parser.parse(data,True)
	elif portdst == 1870:
		if parse_client_packets:
			parser.parse(data,False)



# CAPTURE: sudo tcpdump -i wlan0 -w second.pcap udp port 1870

parser = SalemProtocolParser()
parse_client_packets = False
parse_server_packets = False

if len(argv) != 3:
	print('wrong arguments count')
	exit(1)
if argv[2] == 'client':
	parse_client_packets = True
if argv[2] == 'server':
	parse_server_packets = True
if argv[2] == 'both':
	parse_client_packets = True
	parse_server_packets = True
rdr = pcapy.open_offline(argv[1])
rdr.dispatch(-1,show_info)

resfile = open('resids.txt','wb')
for id in sorted(parser.resids):
	res = parser.resids[id]
	resfile.write('{:5} {:35} {}\n'.format(id,res.name,res.ver).encode())
resfile.close()

objfile = open('objects.txt','wb')
for id in sorted(parser.objdata):
	obj = parser.objdata[id]
	resid = obj.resid
	res = parser.resids.get(resid)
	if res:
		resname = res.name
	else:
		resname = ''
	objfile.write('{} {} {} {} {}\n'.format(obj.fl,obj.frame,obj.coord,resid,resname).encode())
objfile.close()

print()
#for id,buf in parser.fragbufs.items():
#	print('{} {}'.format(id,len(buf)))
parser.parse_fragbufs()
print()

x = None
y = None
for id,obj in parser.objdata.items():
	if obj.coord:
		c = obj.coord
		if not x:
			x = Struct(min=c[0],max=c[0])
			y = Struct(min=c[1],max=c[1])
		else:
			if c[0] < x.min:
				x.min = c[0]
			if c[0] > x.max:
				x.max = c[0]
			if c[1] < y.min:
				y.min = c[1]
			if c[1] > y.max:
				y.max = c[1]

print('x [{}, {}] {}'.format(x.min,x.max,x.max-x.min))
print('y [{}, {}] {}'.format(y.min,y.max,y.max-y.min))
