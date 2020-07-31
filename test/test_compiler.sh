#!/bin/bash

padding_dots=$(printf '%0.1s' "."{1..60})
padlength=50

test_success () {
    echo "OK"
    ((success++))
}

test_failure () {
    echo "FAIL"
    ((fail++))
}

if [ "$1" == "" ]; then
    echo "USAGE: ./test_compiler.sh /path/to/compiler"
    echo "Default: ../target/debug/minidecaf"
    cmp="../target/debug/minidecaf"
else
    cmp=$1
fi



success_total=0
failure_total=0

success=0
fail=0
echo "===================Valid Programs==================="
for prog in `ls *.c 2>/dev/null`; do
    gcc -m32 -w $prog
    expected_out=`./a.out`
    expected_exit_code=$?
    rm a.out

    $cmp $prog >/dev/null
    base="${prog%.*}" #name of executable (filename w/out extension)
    actual_out=`./out`
    actual_exit_code=$?
    rm out *.s
    test_name="${base##*valid/}"
    printf '%s' "$test_name"
    printf '%*.*s' 0 $((padlength - ${#test_name})) "$padding_dots"

    if [[ $test_name == "undefined"* ]]; then
        # return value is undefined
        # make sure it runs w/out segfaulting, but otherwise don't check exit code
        if [ "$actual_exit_code" -eq 139 ]; then
            #segfault!
            test_failure
        else
            test_success
        fi
    else
        # make sure exit code is correct
        if [ "$expected_exit_code" -ne "$actual_exit_code" ] || [ "$expected_out" != "$actual_out" ]
        then
            test_failure
        else
            test_success
        fi
    fi
            
done

printf "%d successes, %d failures\n" $success $fail
