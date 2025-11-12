import json
import uuid
import tarfile
import io
from typing import Any, Dict

import nacl.encoding
import nacl.signing
import requests


class MinerSDK:
    def __init__(self, private_key_hex: str, notary_url: str = "http://localhost:3000"):
        self.private_key = nacl.signing.SigningKey(private_key_hex, encoder=nacl.encoding.HexEncoder)
        self.notary_url = notary_url

    def fetch_task(self) -> Dict[str, Any]:
        """Fetches the current task from the Notary Server."""
        response = requests.get(f"{self.notary_url}/api/v1/task")
        response.raise_for_status()
        return response.json()

    def create_package(self, hyperparameters: Dict[str, Any], wasm_blob: bytes) -> bytes:
        """Creates a reproducibility package as a tar.gz archive in memory."""
        # Create an in-memory tar.gz file
        file_obj = io.BytesIO()
        with tarfile.open(fileobj=file_obj, mode="w:gz") as tar:
            # Add hyperparameters.json
            hyperparameters_bytes = json.dumps(hyperparameters, indent=2).encode("utf-8")
            tarinfo = tarfile.TarInfo(name="hyperparameters.json")
            tarinfo.size = len(hyperparameters_bytes)
            tar.addfile(tarinfo, io.BytesIO(hyperparameters_bytes))

            # Add train.wasm
            wasm_tarinfo = tarfile.TarInfo(name="train.wasm")
            wasm_tarinfo.size = len(wasm_blob)
            tar.addfile(wasm_tarinfo, io.BytesIO(wasm_blob))

        file_obj.seek(0)
        return file_obj.getvalue()

    def sign_payload(self, payload: Dict[str, Any]) -> str:
        """Signs a payload with the miner's private key."""
        payload_bytes = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
        signed = self.private_key.sign(payload_bytes)
        return signed.signature.hex()

    def submit_claim(
        self,
        miner_id: str,
        task_id: str,
        claimed_score: float,
        artifact_hash: str,
        timestamp: str,
        nonce: str,
        artifact: bytes,
    ) -> Dict[str, Any]:
        """Submits a claim to the Notary Server."""
        payload = {
            "miner_id": miner_id,
            "task_id": task_id,
            "claimed_score": claimed_score,
            "artifact_hash": artifact_hash,
            "timestamp": timestamp,
            "nonce": nonce,
        }
        signature = self.sign_payload(payload)

        files = {
            "payload": (None, json.dumps(payload), "application/json"),
            "artifact": ("submission.tar.gz", artifact, "application/gzip"),
        }

        headers = {"X-Signature": signature}
        response = requests.post(
            f"{self.notary_url}/api/v1/submit", files=files, headers=headers
        )
        response.raise_for_status()
        return response.json()
