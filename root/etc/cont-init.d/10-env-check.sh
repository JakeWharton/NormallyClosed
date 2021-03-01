#!/usr/bin/with-contenv sh

if [ -z "$NC_ARGS" ]; then
	echo "
ERROR: 'NC_ARGS' environment variable not set"
fi
