#/bin/sh

for f in memory simple
do
    gcc -o ${f}-native ${f}.c
done
