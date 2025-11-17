tmp-dir:
    @mkdir -p tmp

pycheck-snap case_path: tmp-dir
    scripts/compare_snap.py assets/test_cases/{{case_path}} > tmp/pycheck-{{case_path}}
    difft --exit-code tmp/pycheck-{{case_path}} tests/api/snapshots/api__parse__test_parse_incr-assets__test_cases__{{case_path}}.snap

test-check-snaps: \
    (pycheck-snap "empty-node.conllu") \
    (pycheck-snap "space-after-no.conllu") \
    (pycheck-snap "en_ewt-ud-test_excerp.conllu") \
    (pycheck-snap "paragraph-and-document.conllu") \
    (pycheck-snap "paragraph-in-sentence.conllu") \
    (pycheck-snap "multiword.conllu")
    @echo "All tests passed"

