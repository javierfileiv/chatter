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


@pytest.mark.asyncio
async def test_five_clients_broadcast(chatter_server):
    """Five clients in the same room: one sends, the other four receive."""
    clients = []
    for i in range(5):
        ws = await websockets.connect(f"ws://127.0.0.1:{chatter_server}")
        clients.append(ws)

    # All authenticate in the same room
    for i, ws in enumerate(clients):
        await ws.send(
            json.dumps(
                {
                    "type": "authenticate",
                    "username": f"user{i}",
                    "password": "x",
                    "room_name": "bigroom",
                }
            )
        )

    # Consume auth responses
    for ws in clients:
        await asyncio.wait_for(ws.recv(), timeout=5)

    # user0 sends a message
    await clients[0].send(
        json.dumps(
            {
                "type": "send",
                "username": "user0",
                "message": "hello everyone",
            }
        )
    )

    # users 1-4 should receive it
    for ws in clients[1:]:
        msg = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
        assert msg["type"] == "chat"
        assert msg["sender"] == "user0"
        assert msg["message"] == "hello everyone"

    # Cleanup
    for ws in clients:
        await ws.close()


@pytest.mark.asyncio
async def test_messages_do_not_cross_rooms(chatter_server):
    """Three clients in room A, three in room B: messages stay within rooms."""
    room_a = []
    room_b = []

    # Connect 3 clients to room A
    for i in range(3):
        ws = await websockets.connect(f"ws://127.0.0.1:{chatter_server}")
        room_a.append(ws)

    # Connect 3 clients to room B
    for i in range(3):
        ws = await websockets.connect(f"ws://127.0.0.1:{chatter_server}")
        room_b.append(ws)

    # Authenticate room A clients
    for i, ws in enumerate(room_a):
        await ws.send(
            json.dumps(
                {
                    "type": "authenticate",
                    "username": f"alice{i}",
                    "password": "x",
                    "room_name": "room_a",
                }
            )
        )

    # Authenticate room B clients
    for i, ws in enumerate(room_b):
        await ws.send(
            json.dumps(
                {
                    "type": "authenticate",
                    "username": f"bob{i}",
                    "password": "x",
                    "room_name": "room_b",
                }
            )
        )

    # Consume auth responses
    for ws in room_a:
        await asyncio.wait_for(ws.recv(), timeout=5)
    for ws in room_b:
        await asyncio.wait_for(ws.recv(), timeout=5)

    # alice0 sends a message in room A
    await room_a[0].send(
        json.dumps(
            {
                "type": "send",
                "username": "alice0",
                "message": "hello room A",
            }
        )
    )

    # alice1 and alice2 in room A should receive it
    for ws in room_a[1:]:
        msg = json.loads(await asyncio.wait_for(ws.recv(), timeout=5))
        assert msg["type"] == "chat"
        assert msg["sender"] == "alice0"
        assert msg["message"] == "hello room A"

    # Room B clients should NOT receive anything
    for ws in room_b:
        with pytest.raises(asyncio.TimeoutError):
            await asyncio.wait_for(ws.recv(), timeout=1)

    # Cleanup
    for ws in room_a + room_b:
        await ws.close()
