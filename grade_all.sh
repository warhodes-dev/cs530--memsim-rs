#!/bin/bash
gsdir="../gradescripts"

cargo build --release

memsim="./target/release/memsim-rs"

for idx in {0..100};
do
    cfgid_raw=$(($idx / 10))
    printf -v cfgid "%02d" $cfgid_raw
    printf -v gsid "%03d" $idx

    trace_file="$gsdir/$gsid.dat"

    if [ -f $trace_file ];
    then
        cat $trace_file | (MEMSIM_CONFIG="$gsdir/$cfgid.config" RUSTFLAGS="-Awarnings" $memsim) > ./.tmp_mysolution.txt
        diff -w $gsdir/$gsid.sol ./.tmp_mysolution.txt > /dev/null
        if [ $? -eq 0 ];
        then
            echo "$gsid Passed!"
        else
            echo "$gsid Failed..."
        fi
    else
        rm ./.tmp_mysolution.txt
        exit
    fi
done

rm ./.tmp_mysolution.txt
