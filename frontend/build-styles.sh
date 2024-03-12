#!/bin/sh

INTERMEDIATE=$(mktemp -t grass.XXXXXXXX)

cleanup () { rm -f "${INTERMEDIATE}"; }
trap cleanup 0

INFILE="${TRUNK_SOURCE_DIR}/style/main.scss"
OUTFILE="${TRUNK_SOURCE_DIR}/style/main.css"

grass "${INFILE}" "${INTERMEDIATE}"

if ! [ -e "${OUTFILE}" ]; then
  mv "${INTERMEDIATE}" "${OUTFILE}"
else
  if ! cmp -s "${INTERMEDIATE}" "${OUTFILE}"; then
    mv "${INTERMEDIATE}" "${OUTFILE}"
  fi
fi
