#!/usr/bin/env bash

set -e

rm -f *_thumbnail.*

for prefix in p0 p3b; do
	for image in ${prefix}_*.jpg; do
		echo "Copying '$image'…"
		imageoptim --no-stats "$image"
		cp -- "$image" "${image%.jpg}_thumbnail.jpg"
	done
done

echo "Resizing thumbnails…"
mogrify -thumbnail 300x *_thumbnail.*
