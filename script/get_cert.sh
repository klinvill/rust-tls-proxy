#!/bin/sh

usage () {
  echo "Usage:"
  echo './get_cert.sh [--port <portnum>] [--out <filename>] or'
  echo './get_cert.sh defaults'
  echo 'Please use defaults if you do not plan to use arguments (this script expects at least one argument).'
  exit 1
}

port=9090
outloc=/usr/local/share/ca-certificates/forum.crt

if [ $# -eq 0 ]
then
  usage
fi

arg=None
for val in $@
do
  if [ $arg = 'None' ]
  then
    arg=$val
  else
    #Don't allow two such args to follow each other (containing --)
    if echo $val | grep -q -e '--' 
    then
      usage
    elif [ $arg = '--port' ]
    then
      port=$val
      arg=None
    elif [ $arg = '--out' ]
    then
      outloc=$val
      arg=None
    fi
  fi
done

echo quit | openssl s_client -showcerts -connect localhost:$port > tmp.txt
grep -e '-----BEGIN CERTIFICATE-----.*?-----END CERTIFICATE-----' tmp.txt > $outloc
#rm tmp.txt

exit 0
