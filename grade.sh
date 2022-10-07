#!/bin/bash
gsdir="../gradescripts"

cargo build --release

memsim="./target/release/memsim-rs"

gsid_raw=$1

cfgid_raw=$(($gsid_raw / 10))
printf -v cfgid "%02d" $cfgid_raw
printf -v gsid "%03d" $gsid_raw

trace_file="$gsdir/$gsid.dat"

if [ -f $trace_file ];
then
    cat $trace_file | (MEMSIM_CONFIG="$gsdir/$cfgid.config" RUSTFLAGS="-Awarnings" $memsim) > ./.tmp_mysolution.txt
    diff -W $(( $(tput cols) - 2 )) -w -y $gsdir/$gsid.sol ./.tmp_mysolution.txt > ./.tmp_diff.txt
    if [ $? -eq 0 ];
    then
        echo "$gsid Passed!"
    else
        cat ./.tmp_diff.txt
        echo "$gsid Failed..."
    fi
else
    rm ./.tmp_mysolution.txt
    exit
fi

rm ./.tmp_mysolution.txt
rm ./.tmp_diff.txt
