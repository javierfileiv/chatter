import os
import pytest


@pytest.fixture
def chatter_server():
    """Return the server port from the PORT environment variable."""
    port = os.environ.get("PORT")
    if not port:
        raise RuntimeError("PORT env var not set. Run tests via scripts/test_integration.sh")
    return int(port)
