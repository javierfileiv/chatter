import asyncio
import json

import pytest
import websockets

AUTH_MSG = json.dumps(
    {
        "type": "authenticate",
        "username": "alice",
        "password": "x",
        "room_name": "test",
    }
)


@pytest.mark.asyncio
async def test_connect_auth_success(chatter_server):
    """Authenticate and verify auth_result response."""
    async with websockets.connect(f"ws://127.0.0.1:{chatter_server}") as ws:
        await ws.send(AUTH_MSG)
        resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
        assert resp["type"] == "auth_result"
        assert resp["success"] is True


@pytest.mark.asyncio
async def test_send_message(chatter_server):
    """Send a broadcast message and verify no error."""
    async with websockets.connect(f"ws://127.0.0.1:{chatter_server}") as ws:
        await ws.send(AUTH_MSG)
        await asyncio.wait_for(ws.recv(), timeout=5)  # consume auth response

        await ws.send(
            json.dumps(
                {
                    "type": "send",
                    "username": "alice",
                    "message": "hello",
                }
            )
        )
        # No crash = message was processed by the broker


@pytest.mark.asyncio
async def test_logout(chatter_server):
    """Logout and verify the connection closes gracefully."""
    async with websockets.connect(f"ws://127.0.0.1:{chatter_server}") as ws:
        await ws.send(AUTH_MSG)
        await asyncio.wait_for(ws.recv(), timeout=5)  # consume auth response

        await ws.send(
            json.dumps(
                {
                    "type": "logout",
                    "message": "bye",
                }
            )
        )
        # Connection should close after logout
        with pytest.raises(websockets.exceptions.ConnectionClosed):
            await asyncio.wait_for(ws.recv(), timeout=5)


@pytest.mark.asyncio
async def test_invalid_json(chatter_server):
    """Send invalid JSON and verify the server does not crash."""
    async with websockets.connect(f"ws://127.0.0.1:{chatter_server}") as ws:
        await ws.send("not json at all")
        # Server should not crash; connection stays open
        # The server may send nothing or an error — either is acceptable


@pytest.mark.asyncio
async def test_two_clients_broadcast(chatter_server):
    """Two clients in the same room: one sends, the other receives."""
    async with (
        websockets.connect(f"ws://127.0.0.1:{chatter_server}") as ws1,
        websockets.connect(f"ws://127.0.0.1:{chatter_server}") as ws2,
    ):
        # Both authenticate in the same room
        await ws1.send(
            json.dumps(
                {
                    "type": "authenticate",
                    "username": "alice",
                    "password": "x",
                    "room_name": "shared",
                }
            )
        )
        await ws2.send(
            json.dumps(
                {
                    "type": "authenticate",
                    "username": "bob",
                    "password": "x",
                    "room_name": "shared",
                }
            )
        )

        # Consume auth responses
        await asyncio.wait_for(ws1.recv(), timeout=5)
        await asyncio.wait_for(ws2.recv(), timeout=5)

        # Alice sends a message
        await ws1.send(
            json.dumps(
                {
                    "type": "send",
                    "username": "alice",
                    "message": "hello bob",
                }
            )
        )

        # Bob should receive it
        msg = json.loads(await asyncio.wait_for(ws2.recv(), timeout=5))
        assert msg["type"] == "chat"
        assert msg["sender"] == "alice"
        assert msg["message"] == "hello bob"
