#!/usr/bin/with-contenv sh

if [ -z "$API_SECRET" ]; then
	echo "
ERROR: 'GARAGE_PIE_ARGS' environment variable not set"
fi
