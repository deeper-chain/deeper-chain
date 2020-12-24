#!/bin/bash

keyword=$1
for pid in `ps -ef |ag $keyword |awk '{print $2}'`;do
	echo "killing thread ...", $pid
	sudo kill -9 $pid || true
done
