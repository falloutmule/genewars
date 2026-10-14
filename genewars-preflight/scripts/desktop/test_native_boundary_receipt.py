"""No-window checks for native blocked receipt classification."""
import json,tempfile,unittest
from pathlib import Path
from validate_native_building_chain import read_stopped_boundary,assert_expected_boundary
class BoundaryReceiptTests(unittest.TestCase):
    def test_missing_receipt_is_not_success(self):
        with tempfile.TemporaryDirectory() as d:self.assertIsNone(read_stopped_boundary(Path(d)))
    def test_exact_boundary_and_mismatch(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);reason='Family8Motion(UnsupportedRecord)'
            (p/'__outcome.json').write_text(json.dumps(dict(status='blocked',reason=f'native input replay blocked at step 1137: {reason}')))
            (p/'__blocked.json').write_text(json.dumps(dict(mission=dict(blocked=reason,steps=1137,world_sha256='a'*64))))
            result=read_stopped_boundary(p);assert_expected_boundary(result,reason,1137)
            with self.assertRaises(AssertionError):assert_expected_boundary(result,reason,1138)
            with self.assertRaises(AssertionError):assert_expected_boundary(result,'other',1137)
            (p/'__outcome.json').write_text(json.dumps(dict(status='blocked',reason='wrong')))
            with self.assertRaises(ValueError):read_stopped_boundary(p)
if __name__=='__main__':unittest.main()
