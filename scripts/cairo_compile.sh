#!/bin/bash

echo "Compiling Cairo Program"
cairo-compile --cairo_path=cairo/packages/garaga_zero/src "cairo/src/main.cairo" --output "cairo/build/main.json"


if [ $? -eq 0 ]; then
    echo "Compilation Successful!"
    
fi
