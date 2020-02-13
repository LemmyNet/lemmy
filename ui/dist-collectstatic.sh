#!/bin/bash

cd dist
mkdir static 2>/dev/null
mv *.js static 2>/dev/null
mv assets static 2>/dev/null
cd ..
