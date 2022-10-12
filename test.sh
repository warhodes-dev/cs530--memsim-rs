#!/bin/bash

cat > ./.tmp_trace.dat
trace_file=./.tmp_trace.dat

diff --color -w <(cat $trace_file | ../memhier/memhier_ref | head -n -26) <(cat $trace_file | RUSTFLAGS="-Awarnings" cargo run)

if [ $? -eq 0 ] 
then
    echo "Test passed!"
fi

rm ./.tmp_trace.log
