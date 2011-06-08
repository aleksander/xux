#!/usr/bin/env python
# -*- coding: utf-8 -*-

from setuptools import setup, find_packages
setup(
	name = "pure-pcapy",
	version = "1.0.1",
	packages = find_packages(exclude=['test'],),

	author = "Stanislaw Pitucha",
	author_email = "viraptor@gmail.com",
	description = "Pure Python reimplementation of pcapy. This package is API compatible and a drop-in replacement.",
	license = "Simplified BSD",
	keywords = ["pcap", "file", "reader", "writer"],
	url = "http://bitbucket.org/viraptor/pure-pcapy/overview",

	long_description = "This package provides an API-compatible replacement for the popular pcapy package. Since it is a pure-Python package, it cannot access some elements like live traffic capture directly. Only file operations are allowed right now. Any operation which is available in pcapy and could not be implemented here will throw NotImplementedError. Behaviour should be compatible up to the text of some exceptions (tests are included).",

	install_requires=['setuptools'],
	test_suite = "test",
)

