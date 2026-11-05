import ctypes, json, os, socketserver, ssl, subprocess, threading, winreg
from pathlib import Path
if os.environ.get('GITHUB_ACTIONS')!='true' or os.environ.get('RUNNER_OS')!='Windows' or os.environ.get('RUNNER_ENVIRONMENT')!='github-hosted':
 raise SystemExit('Only a disposable GitHub-hosted Windows runner is allowed')
root=Path.cwd(); out=root/'build/issue165-system-proxy'; out.mkdir(exist_ok=False)
cert=out/'mirror-cert.pem';key=out/'mirror-private.pem'
openssl=Path('C:/Program Files/Git/usr/bin/openssl.exe')
subprocess.run([str(openssl),'req','-x509','-newkey','rsa:2048','-nodes','-days','2','-keyout',str(key),'-out',str(cert),'-subj','/CN=mirror.example.test','-addext','subjectAltName=DNS:mirror.example.test','-addext','basicConstraints=critical,CA:TRUE','-addext','keyUsage=critical,digitalSignature,keyEncipherment,keyCertSign','-addext','extendedKeyUsage=serverAuth'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True)
context=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);context.load_cert_chain(cert,key)
fixture=root/'tests/fixtures/update';manifest=json.loads((fixture/'network-manifest.json').read_text(encoding='utf-8'))
assets=[{'name':a['filename'],'size':4,'browser_download_url':a['url']} for a in manifest['assets']]
base='https://github.com/xiaodaozhi/xdzip/releases/download/v9.0.0/'
assets.extend([{'name':'release.json','browser_download_url':base+'release.json'},{'name':'release.json.sig','browser_download_url':base+'release.json.sig'}])
listing=json.dumps([{'tag_name':'v9.0.0','draft':False,'assets':assets}]).encode()
events=[];lock=threading.Lock()
class Proxy(socketserver.ThreadingTCPServer):
 allow_reuse_address=True
 daemon_threads=True
class Handler(socketserver.BaseRequestHandler):
 def handle(self):
  raw=self.request;raw.settimeout(20)
  try:
   header=b''
   while b'\r\n\r\n' not in header and len(header)<16384:
    part=raw.recv(4096)
    if not part:return
    header+=part
   first=header.split(b'\r\n',1)[0]
   if first!=b'CONNECT mirror.example.test:443 HTTP/1.1':raise ValueError('Unexpected proxy target')
   raw.sendall(b'HTTP/1.1 200 Connection Established\r\n\r\n')
   with context.wrap_socket(raw,server_side=True) as channel:
    request=b''
    while b'\r\n\r\n' not in request and len(request)<16384:
     chunk=channel.recv(4096)
     if not chunk:return
     request+=chunk
    pieces=request.split(b'\r\n',1)[0].split(b' ')
    if len(pieces)!=3 or pieces[0]!=b'GET':raise ValueError('Unexpected method')
    path=pieces[1].decode('ascii')
    if path.startswith('/prefix/https://api.github.com/repos/xiaodaozhi/xdzip/releases?'):
     kind='release-list';content=listing
    elif path=='/prefix/'+base+'release.json':kind='manifest';content=(fixture/'network-manifest.json').read_bytes()
    elif path=='/prefix/'+base+'release.json.sig':kind='signature';content=(fixture/'network-manifest.json.sig').read_bytes()
    elif path.startswith('/prefix/'+base+'xdzip-9.0.0-'):kind='package';content=b'data'
    else:raise ValueError('Unexpected mirror path')
    with lock:events.append({'kind':kind,'tls':channel.version()})
    channel.sendall(b'HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: '+str(len(content)).encode()+b'\r\nConnection: close\r\n\r\n'+content)
  except (OSError,ValueError,ssl.SSLError) as e:
   with lock:events.append({'error_type':type(e).__name__})
with Proxy(('127.0.0.1',0),Handler) as server:
 thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
 env=os.environ.copy();env['PATH']=str(Path(os.environ['RUNNER_TEMP'])/'Qt/6.8.2/msvc2022_64/bin')+os.pathsep+env['PATH'];env['QT_QPA_PLATFORM']='offscreen';env['QT_QUICK_BACKEND']='software';env['XDZIP_ISSUE165_MIRROR_PROXY']='';env['XDZIP_ISSUE165_MIRROR_MODE']='system';env['XDZIP_ISSUE165_MIRROR_CA']=str(cert);env['XDZIP_ISSUE165_MIRROR_NEGATIVE']=''
 exe=root/'build/issue165/apps/desktop/xdzip-update-tests.exe'
 names=('ProxyEnable','ProxyServer','ProxyOverride','AutoConfigURL','AutoDetect')
 with winreg.OpenKey(winreg.HKEY_CURRENT_USER, r'Software\Microsoft\Windows\CurrentVersion\Internet Settings', 0, winreg.KEY_QUERY_VALUE|winreg.KEY_SET_VALUE) as registry:
  def snapshot():
   values={}
   for name in names:
    try:values[name]=winreg.QueryValueEx(registry,name)
    except FileNotFoundError:values[name]=None
   return values
  def refresh():
   for option in (39,37):
    if not ctypes.windll.wininet.InternetSetOptionW(None,option,None,0):raise OSError('Unable to refresh system proxy settings')
  before=snapshot()
  try:
   winreg.SetValueEx(registry,'ProxyEnable',0,winreg.REG_DWORD,1)
   winreg.SetValueEx(registry,'ProxyServer',0,winreg.REG_SZ,f'127.0.0.1:{server.server_address[1]}')
   winreg.SetValueEx(registry,'ProxyOverride',0,winreg.REG_SZ,'')
   winreg.SetValueEx(registry,'AutoDetect',0,winreg.REG_DWORD,0)
   try:winreg.DeleteValue(registry,'AutoConfigURL')
   except FileNotFoundError:pass
   refresh()
   result=subprocess.run([str(exe),'temporaryMirrorTlsProbe','-o',str(out/'qt-test.xml')+',xml'],env=env,timeout=90,creationflags=subprocess.CREATE_NO_WINDOW)
  finally:
   for name,value in before.items():
    if value is None:
     try:winreg.DeleteValue(registry,name)
     except FileNotFoundError:pass
    else:winreg.SetValueEx(registry,name,0,value[1],value[0])
   refresh()
   if snapshot()!=before:raise RuntimeError('System proxy settings did not restore')
   key.unlink(missing_ok=True)
 server.shutdown();thread.join(timeout=5)
record={'exit_code':result.returncode,'requests':events,'scope':'disposable Windows system proxy and isolated TLS mirror with fixture-only CA and signing trust'}
(out/'network.json').write_text(json.dumps(record,indent=2),encoding='utf-8');print(json.dumps(record))
print((out/'qt-test.xml').read_text(encoding='utf-8')[-2500:])

if result.returncode or [item.get('kind') for item in events]!=['release-list','manifest','signature','package']:
 raise SystemExit(1)
