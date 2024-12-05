#!/usr/bin/env sh

# Wrapper script to handle automatic updates.
# When status 169 is returned can assume a new release has been downloaded and unpack it.
# The script assumes it's placed in the same directory as the rust binary for distribution.

cd $(dirname $0)

name=bol-scraper-empire
archive_name=$name.latest.tar.gz

function scrape() {
  ./$name scrape --ask-location
}

scrape
return_value=$?

if [ $return_value -eq 169 ]; then
  if [[ -f $archive_name ]]; then
    temp_dir=$(mktemp -d)
    tar xf $archive_name --directory $temp_dir
    rm $archive_name
    cp -v $temp_dir/* ./
    rm -rf $temp_dir 

    scrape
  else
    echo Er is iets fout gegaan tijdens het updaten
    echo Programma gaf een update status code maar er is geen archief gevonden van de laatste release.
  fi
fi

