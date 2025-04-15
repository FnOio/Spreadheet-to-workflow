#!/usr/bin/env sh

# Arguments:
# - path to spreadsheet containing an OSLO-steps workflow, in ODS format.

# This script requires:
# - java >= 17
# - node.js >= 18
# - YARRRML parser installed
# - RMLMapper installed
# - spreadsheet-to-workflow installed
# TODO: make Docker image that contains all this stuff

# !!MAKE THESE VARIABLES POINT TO THE RIGHT PATHS!!
YARRRMLPARSER=/home/geraldh/projects/yarrrml/yarrrml-parser/bin/parser.js
RMLMAPPER=/home/geraldh/projects/rml/rmlmapper-java/target/rmlmapper-7.3.3-r0-all.jar
STW=/home/geraldh/projects/oslo/prepare-flow-mappings/target/release/spreadsheet-to-flow

if [ $# -eq 0 ]
  then
    echo "Usage: ./run.sh <path-to-flow.ods>"
    exit 1
fi

# Exit if there are errors
set -e

flowfile=$1
echo "Extracting O-STEPS compatible flow from $FLOWFILE"
$STW $flowfile

# Find output directory of flow data and mappings
flow_output_dir="${flowfile%.*}_output"

echo 'Turning YARRRML mappings to RML mappings...'
yarrrmlmapping=${flow_output_dir}/mapping.yarrrml.yaml
rmlmapping=${flow_output_dir}/mapping.rml.ttl 
$YARRRMLPARSER -p -m -i $yarrrmlmapping -o $rmlmapping

echo 'Turning the flow data into RDF (using RMLMapper)...'
java -jar $RMLMAPPER --mappingfile $rmlmapping

echo "Done! Written everything to $flow_output_dir"
