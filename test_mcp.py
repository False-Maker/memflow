import subprocess
import json
import time
import sys

def test_mcp():
    process = subprocess.Popen(
        ['cargo', 'run', '-p', 'memflow-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding='utf-8',
        bufsize=1,
        cwd='d:/Demo/memflow'
    )

    # Helper to send request
    def send(req):
        process.stdin.write(json.dumps(req) + '\n')
        process.stdin.flush()

    # 1. Initialize
    send({"jsonrpc": "2.0", "method": "initialize", "params": {"capabilities": {}}, "id": 1})
    print("Sent initialize")
    
    # Wait for DB init and Model download
    print("Waiting for model and DB init (watching stderr)...")
    
    # Simple loop to read stderr and wait for completion
    import threading
    def stream_stderr():
        for line in iter(process.stderr.readline, ''):
            print(f"ERR: {line.strip()}", file=sys.stderr)
    
    t = threading.Thread(target=stream_stderr)
    t.daemon = True
    t.start()

    time.sleep(30) # Wait 30s for download/init

    # 2. Search
    send({"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "search_memory", "arguments": {"query": "test", "limit": 2}}, "id": 2})
    print("Sent search")

    # Read response
    while True:
        line = process.stdout.readline()
        if not line:
            break
        print(f"OUT: {line.strip()}")
        try:
            res = json.loads(line)
            if res.get("id") == 2:
                break
        except:
            pass

    process.terminate()
    
    # Print stderr for debugging
    err = process.stderr.read()
    print(f"ERR: {err}")

if __name__ == "__main__":
    test_mcp()
