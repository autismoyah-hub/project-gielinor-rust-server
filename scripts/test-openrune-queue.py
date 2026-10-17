import copy,importlib.util,json,unittest
from pathlib import Path
r=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('queue_validator',r/'scripts/validate-openrune-queue.py');validator=importlib.util.module_from_spec(spec);spec.loader.exec_module(validator)
class QueueTests(unittest.TestCase):
 def setUp(self):self.record=json.loads((r/'docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()[0])
 def test_schema_required_fields_match_validator(self):
  schema=json.loads((r/'docs/openrune/verification/IMPLEMENTATION_QUEUE.schema.json').read_text());self.assertEqual(set(schema['required']),validator.REQUIRED);self.assertEqual(set(schema['properties']),validator.REQUIRED)
 def test_valid_contract(self):self.assertEqual(validator.validate([self.record]),1)
 def test_missing_contract_and_identity_fields_fail(self):
  for field in validator.REQUIRED:
   value=copy.deepcopy(self.record);del value[field]
   with self.assertRaises(ValueError):validator.validate([value])
 def test_bad_sha_or_independent_verdict_fails(self):
  for field,value in [('implementation_commit_sha','HEAD'),('verification_status','PASS'),('implementation_status','stub')]:
   record=copy.deepcopy(self.record);record[field]=value
   with self.assertRaises(ValueError):validator.validate([record])
 def test_legacy_evidence_cannot_be_relabelled_independent(self):
  self.record['legacy_evidence'][0]['independent_verification']=True
  with self.assertRaises(ValueError):validator.validate([self.record])
 def test_duplicate_first_revision_and_unknown_dependency_fail(self):
  with self.assertRaises(ValueError):validator.validate([self.record,copy.deepcopy(self.record)])
  self.record['dependencies'].append('OR-MIG-9999')
  with self.assertRaises(ValueError):validator.validate([self.record])
 def test_append_revision_requires_exact_previous_sha(self):
  next_record=copy.deepcopy(self.record);next_record['revision']=2;next_record['supersedes']={'revision':1,'implementation_commit_sha':self.record['implementation_commit_sha']}
  self.assertEqual(validator.validate([self.record,next_record]),1);next_record['supersedes']['implementation_commit_sha']='0'*40
  with self.assertRaises(ValueError):validator.validate([self.record,next_record])
 def test_dependency_cycle_fails(self):
  other=copy.deepcopy(self.record);other['implementation_id']='OR-MIG-9999';other['dependencies']=[self.record['implementation_id']];self.record['dependencies']=[other['implementation_id']]
  with self.assertRaises(ValueError):validator.validate([self.record,other])
 def test_levels_and_paths_are_complete_and_safe(self):
  for mutate in [lambda r:r['required_verification_levels'].pop(),lambda r:r['required_verification_levels'][0].update(required=False),lambda r:r['rust_paths'].append('../outside.rs')]:
   record=copy.deepcopy(self.record);mutate(record)
   with self.assertRaises(ValueError):validator.validate([record])
if __name__=='__main__':unittest.main()
