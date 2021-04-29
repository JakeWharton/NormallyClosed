#!/usr/bin/env bash

set -e

rm -f *_thumbnail.*

for prefix in p0 p3b; do
	for extension in jpg; do
		for image in ${prefix}_*.$extension; do
			echo "Copying '$image'…"
			cp -- "$image" "${image%.$extension}_thumbnail.$extension"
		done
	done
done

echo "Resizing thumbnails…"
mogrify -thumbnail 300x *_thumbnail.*
