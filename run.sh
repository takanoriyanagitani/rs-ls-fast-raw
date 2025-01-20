#!/bin/sh

./rs-ls-fast-raw |
	sed \
		-n \
		-e '/^\./d' \
		-e p |
	sort
