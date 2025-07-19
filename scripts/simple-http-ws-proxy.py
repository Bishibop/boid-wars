#!/usr/bin/env python3
"""
Simple HTTP server that serves static files and proxies WebSocket connections.
This allows us to run everything on a single port in production.
"""
import http.server
import socketserver
import socket
import select
import os
import sys

class HTTPWebSocketHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        # Serve files from /app/static
        super().__init__(*args, directory="/app/static", **kwargs)
    
    def do_GET(self):
        # Check if this is a WebSocket upgrade request
        if self.headers.get('Upgrade', '').lower() == 'websocket':
            self.handle_websocket()
        else:
            # Normal HTTP request - serve static files
            super().do_GET()
    
    def handle_websocket(self):
        """Forward WebSocket connection to the game server running on localhost:8081"""
        try:
            # Connect to the game server
            backend = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            backend.connect(('127.0.0.1', 8081))
            
            # Forward the original request
            request_line = f"{self.command} {self.path} {self.request_version}\r\n"
            backend.send(request_line.encode())
            
            # Forward all headers
            for header in self.headers:
                header_line = f"{header}: {self.headers[header]}\r\n"
                backend.send(header_line.encode())
            backend.send(b"\r\n")
            
            # Get the client socket
            client = self.request
            
            # Set both sockets to non-blocking
            client.setblocking(0)
            backend.setblocking(0)
            
            # Relay data between client and backend
            sockets = [client, backend]
            while sockets:
                readable, _, exceptional = select.select(sockets, [], sockets, 1.0)
                
                for s in exceptional:
                    sockets.remove(s)
                    s.close()
                
                for s in readable:
                    try:
                        data = s.recv(4096)
                        if data:
                            # Forward to the other socket
                            other = backend if s is client else client
                            other.sendall(data)
                        else:
                            # Socket closed
                            sockets.remove(s)
                            s.close()
                            if other in sockets:
                                sockets.remove(other)
                                other.close()
                    except socket.error:
                        # Error on socket
                        if s in sockets:
                            sockets.remove(s)
                            s.close()
            
            # Prevent the base handler from closing the connection
            self.close_connection = False
            
        except Exception as e:
            print(f"WebSocket proxy error: {e}")
            self.send_error(502, "Bad Gateway")
    
    def log_message(self, format, *args):
        # Only log non-asset requests to reduce noise
        if not any(self.path.startswith(p) for p in ['/pkg/', '/assets/', '.js', '.wasm']):
            sys.stderr.write("%s - - [%s] %s\n" %
                            (self.client_address[0],
                             self.log_date_time_string(),
                             format%args))

if __name__ == "__main__":
    PORT = 8080
    
    # Allow socket reuse
    socketserver.TCPServer.allow_reuse_address = True
    
    with socketserver.TCPServer(("", PORT), HTTPWebSocketHandler) as httpd:
        print(f"🌐 HTTP/WebSocket proxy server running on port {PORT}")
        print(f"   - Serving static files from /app/static")
        print(f"   - Proxying WebSocket to localhost:8081")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n🛑 Server stopped")