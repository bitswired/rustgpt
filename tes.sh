#!/bin/sh
just dev-tailwind &
pid1=$!
just dev-tailwind &
pid2=$!
trap "kill $pid1 $pid2" EXIT
wait $pid1 $pid2

