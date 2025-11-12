import unittest
from unittest.mock import patch, MagicMock
from miner_sdk.client import MinerSDK
import tarfile
import io
import json

class TestMinerSDK(unittest.TestCase):
    def setUp(self):
        self.sdk = MinerSDK(private_key_hex="0" * 64)

    @patch("requests.get")
    def test_fetch_task(self, mock_get):
        mock_response = MagicMock()
        mock_response.json.return_value = {"task_id": "test_task"}
        mock_get.return_value = mock_response

        task = self.sdk.fetch_task()
        self.assertEqual(task["task_id"], "test_task")
        mock_get.assert_called_once_with(f"{self.sdk.notary_url}/api/v1/task")

    def test_create_package(self):
        hyperparameters = {"param1": "value1"}
        wasm_blob = b"test_wasm_blob"

        package = self.sdk.create_package(hyperparameters, wasm_blob)

        with tarfile.open(fileobj=io.BytesIO(package), mode="r:gz") as tar:
            names = tar.getnames()
            self.assertIn("hyperparameters.json", names)
            self.assertIn("train.wasm", names)

            hyperparameters_file = tar.extractfile("hyperparameters.json")
            self.assertIsNotNone(hyperparameters_file)
            hyperparameters_data = json.load(hyperparameters_file)
            self.assertEqual(hyperparameters_data, hyperparameters)

            wasm_file = tar.extractfile("train.wasm")
            self.assertIsNotNone(wasm_file)
            wasm_data = wasm_file.read()
            self.assertEqual(wasm_data, wasm_blob)

    @patch("requests.post")
    def test_submit_claim(self, mock_post):
        mock_response = MagicMock()
        mock_response.json.return_value = {"status": "pending_verification"}
        mock_post.return_value = mock_response

        response = self.sdk.submit_claim(
            miner_id="test_miner",
            task_id="test_task",
            claimed_score=0.99,
            artifact_hash="test_hash",
            timestamp="2025-01-01T12:00:00Z",
            nonce="test_nonce",
            artifact=b"test_artifact",
        )
        self.assertEqual(response["status"], "pending_verification")

if __name__ == "__main__":
    unittest.main()
