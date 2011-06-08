#!/usr/bin/env python
# -*- coding: utf-8 -*-

import unittest, tempfile, struct
import pure_pcapy

class OpenerTest(unittest.TestCase):
	def test_open_nonexistant(self):
		self.assertRaises(pure_pcapy.PcapError, pure_pcapy.open_offline, 'hufioewhauiefhshcuidhuighureiahufiesa')
	def test_open_directory(self):
		self.assertRaises(pure_pcapy.PcapError, pure_pcapy.open_offline, './')
	
	def test_open_good_file(self):
		import os
		input_filename = tempfile.mktemp()
		f = open(input_filename, "wb")
		f.write(struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1))
		f.close()
		reader = pure_pcapy.open_offline(input_filename)
		self.assertTrue(isinstance(reader, pure_pcapy.Reader))
		os.unlink(input_filename)

class FixupTest(unittest.TestCase):
	def test_fixup_short(self):
		self.assertEquals(0x3412, pure_pcapy.fixup_swapped_short(0x1234))
	def test_fixup_long(self):
		self.assertEquals(0x78563412, pure_pcapy.fixup_swapped_long(0x12345678))

def create_pcap_file(parts):
	input = tempfile.TemporaryFile()
	for part in parts:
		input.write(part)
	input.seek(0)
	return input

class ReaderHeaderTest(unittest.TestCase):
	def test_open_short(self):
		input = tempfile.TemporaryFile()
		try:
			pure_pcapy.Reader(input)
			self.fail("exception not thrown")
		except pure_pcapy.PcapError as e:
			self.assertEqual("truncated dump file; tried to read 24 file header bytes, only got 0", e.args[0])
	
	def test_open_header_bad(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0x11111111, 2, 4, 0, 0, 65535, 1)
			])

		try:
			pure_pcapy.Reader(input)
			self.fail("exception not thrown")
		except pure_pcapy.PcapError as e:
			self.assertEqual("bad dump file format", e.args[0])
	
	def test_open_header_bigendian(self):
		input = create_pcap_file([
			struct.pack(">IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1)
			])

		reader = pure_pcapy.Reader(input)
		self.assertEqual(2, reader.version_major)
		self.assertEqual(4, reader.version_minor)
	
	def test_open_header_littleendian(self):
		input = create_pcap_file([
			struct.pack("<IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1)
			])

		reader = pure_pcapy.Reader(input)
		self.assertEqual(2, reader.version_major)
		self.assertEqual(4, reader.version_minor)
	
	def test_datalink(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1234)
			])

		reader = pure_pcapy.Reader(input)
		self.assertEqual(1234, reader.datalink())

	def test_getnonblock(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1234)
			])

		reader = pure_pcapy.Reader(input)
		self.assertEqual(0, reader.getnonblock())

class ReaderPacketTest(unittest.TestCase):
	def test_read_empty(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			])

		reader = pure_pcapy.Reader(input)
		res = reader.next()
		self.assertEqual((None, ''), res)
	
	def test_read_half_header(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("II", 10, 20),
			])

		reader = pure_pcapy.Reader(input)
		try:
			res = reader.next()
			self.fail("exception not thrown")
		except pure_pcapy.PcapError as e:
			self.assertEqual("truncated dump file; tried to read 16 header bytes, only got 8", e.args[0])
	
	def test_read_half_data(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 4, 4),
			b"aa"
			])

		reader = pure_pcapy.Reader(input)
		try:
			res = reader.next()
			self.fail("exception not thrown")
		except pure_pcapy.PcapError as e:
			self.assertEqual("truncated dump file; tried to read 4 captured bytes, only got 2", e.args[0])

	def test_read_packet(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 30, 40),
			b"x" * 30
			])

		reader = pure_pcapy.Reader(input)
		res = reader.next()
		self.assertEqual(tuple, type(res))
		self.assertEqual(2, len(res))
		self.assertTrue(isinstance(res[0], pure_pcapy.Pkthdr))
		self.assertEqual(bytes, type(res[1]))
		self.assertEqual(b"x"*30, res[1])

	def test_read_2_packets(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 30, 40),
			b"a" * 30,
			struct.pack("IIII", 11, 20, 30, 40),
			b"b" * 30,
			])

		reader = pure_pcapy.Reader(input)
		res = reader.next()
		self.assertEqual(b"a"*30, res[1])
		res = reader.next()
		self.assertEqual(b"b"*30, res[1])

class ReaderLoopingTest(unittest.TestCase):
	def packet_counter(self, cnt):
		def callback(hdr, data):
			cnt[0]+=1
			self.assertEqual(cnt[0], hdr.getcaplen())
			self.assertEqual(cnt[0], len(data))
		return callback

	def test_dispatch_no_packets(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			])

		cnt = [0]
		reader = pure_pcapy.Reader(input)
		res = reader.dispatch(-1, self.packet_counter(cnt))
		self.assertEqual(0, cnt[0])
		self.assertEqual(0, res)
		res = reader.dispatch(0, self.packet_counter(cnt))
		self.assertEqual(0, cnt[0])
		self.assertEqual(0, res)
	
	def test_loop_no_packets(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			])

		cnt = [0]
		reader = pure_pcapy.Reader(input)
		res = reader.loop(0, self.packet_counter(cnt))
		self.assertEqual(0, cnt[0])
		self.assertEqual(None, res)
		res = reader.loop(0, self.packet_counter(cnt))
		self.assertEqual(0, cnt[0])
		self.assertEqual(None, res)
	
	def test_dispatch_1_packet_read_all(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 1, 1),
			b"a",
			])

		cnt = [0]
		reader = pure_pcapy.Reader(input)
		res = reader.dispatch(-1, self.packet_counter(cnt))
		self.assertEqual(1, cnt[0])
		self.assertEqual(0, res)
	
	def test_loop_1_packet_read_all(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 1, 1),
			b"a",
			])

		cnt = [0]
		reader = pure_pcapy.Reader(input)
		res = reader.loop(-1, self.packet_counter(cnt))
		self.assertEqual(1, cnt[0])
		self.assertEqual(None, res)
	
	def test_dispatch_1_packet_read_1(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 1, 1),
			b"a",
			])

		cnt = [0]
		reader = pure_pcapy.Reader(input)
		res = reader.dispatch(1, self.packet_counter(cnt))
		self.assertEqual(1, cnt[0])
		self.assertEqual(1, res)
	
	def test_loop_1_packet_read_1(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 1, 1),
			b"a",
			])

		cnt = [0]
		reader = pure_pcapy.Reader(input)
		res = reader.loop(1, self.packet_counter(cnt))
		self.assertEqual(1, cnt[0])
		self.assertEqual(None, res)
	
	def test_dispatch_2_packet_read_1(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 1, 1),
			b"a",
			struct.pack("IIII", 10, 20, 2, 2),
			b"aa",
			])

		cnt = [0]
		reader = pure_pcapy.Reader(input)
		res = reader.dispatch(1, self.packet_counter(cnt))
		self.assertEqual(cnt[0], 1)
		self.assertEqual(res, 1)
	
	def test_loop_2_packet_read_1(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 1, 1),
			b"a",
			struct.pack("IIII", 10, 20, 2, 2),
			b"aa",
			])

		cnt = [0]
		reader = pure_pcapy.Reader(input)
		res = reader.loop(1, self.packet_counter(cnt))
		self.assertEqual(cnt[0], 1)
		self.assertEqual(res, None)

class PkthdrTest(unittest.TestCase):
	def test_hdr_fields(self):
		input = create_pcap_file([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 30, 40),
			b"x" * 30
			])
		
		reader = pure_pcapy.Reader(input)
		hdr, data = reader.next()
		self.assertEqual((10, 20), hdr.getts())
		self.assertEqual(30, hdr.getcaplen())
		self.assertEqual(40, hdr.getlen())

class DumperTest(unittest.TestCase):
	def test_replicate(self):
		import os

		original = b''.join([
			struct.pack("IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1),
			struct.pack("IIII", 10, 20, 30, 40),
			b"x" * 30])

		input = tempfile.TemporaryFile()
		input.write(original)
		input.seek(0)
		output_fd, output_filename = tempfile.mkstemp()

		reader = pure_pcapy.Reader(input)
		dumper = reader.dump_open(output_filename)
		dumper.dump(*reader.next())

		os.close(output_fd)
		verify = open(output_filename, "rb")
		replica = verify.read()
		os.unlink(output_filename)

		self.assertEqual(original, replica)


if __name__ == "__main__":
	unittest.main()
