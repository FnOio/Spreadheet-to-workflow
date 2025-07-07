#!/usr/bin/env bash

args="$@"
echo "arguments: $args"
if [ $# -ne 1 ]
then
	echo 'Arguments: <path-to-flow.ods>'
	exit 1
fi

# Exit if there are errors
set -e

flowfile=$1
echo "Extracting O-STEPS compatible flow from $FLOWFILE"

./spreadsheet-to-flow /mnt/data/$flowfile

# Find output directory of flow data and mappings
flow_output_dir="/mnt/data/${flowfile%.*}_output"

echo 'Turning YARRRML mappings to RML mappings...'
yarrrmlmapping=${flow_output_dir}/mapping.yarrrml.yaml
rmlmapping=${flow_output_dir}/mapping.rml.ttl 
yarrrml-parser -p -m -i $yarrrmlmapping -o $rmlmapping

echo 'Turning the flow data into RDF (using RMLMapper)...'
java -jar rmlmapper.jar --mappingfile $rmlmapping

echo "Done! Written everything to $flow_output_dir"
