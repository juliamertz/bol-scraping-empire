#!/usr/bin/env sh

name=bol-scraper-empire
archive_name=$name.latest.tar.gz

./$name
return_value=$?

if [ $return_value -eq 169 ]; then
  if [[ -f $archive_name ]]; then
    temp_dir=$(mktemp -d)
    tar xf $archive_name --directory $temp_dir
    rm $archive_name
    cp -v $temp_dir/* ./
    rm -rf $temp_dir

    ./$name
  else
    echo Er is iets fout gegaan tijdens het updaten
    echo Programma gaf een update status code maar er is geen archief gevonden van de laatste release.
  fi
fi

