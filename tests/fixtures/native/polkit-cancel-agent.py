# tests/fixtures/native/polkit-cancel-agent.py

"""Dismiss a genuine polkit challenge in an explicitly disposable Linux test container."""
import argparse
import ctypes as C
import ctypes.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time

AGENT = b'org.freedesktop.PolicyKit1.AuthenticationAgent'
AUTHORITY = b'org.freedesktop.PolicyKit1.Authority'
PATH = b'/org/xdzip/OwnedCancelAgent'
XML = b'''<node><interface name="org.freedesktop.PolicyKit1.AuthenticationAgent">
<method name="BeginAuthentication"><arg type="s" direction="in"/><arg type="s" direction="in"/>
<arg type="s" direction="in"/><arg type="a{ss}" direction="in"/><arg type="s" direction="in"/>
<arg type="a(sa{sv})" direction="in"/></method>
<method name="CancelAuthentication"><arg type="s" direction="in"/></method>
</interface></node>'''


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--request',type=Path,required=True)
    parser.add_argument('--record',type=Path)
    parser.add_argument('--invoke-fd',type=int)
    args=parser.parse_args()
    if sys.platform!='linux' or os.getuid()!=19731 or not Path('/.dockerenv').exists() or os.environ.get('XDZIP_ISOLATED_UPDATE_FIXTURE')!='1':
        parser.error('Only the owned fixture UID in the marked disposable container is supported')
    if args.invoke_fd is not None:
        token=os.read(args.invoke_fd,1);os.close(args.invoke_fd)
        if token!=b'1':raise RuntimeError('Authorization agent did not become ready')
        os.execv('/usr/libexec/xdzip-update-helper',['/usr/libexec/xdzip-update-helper','--authorize',str(args.request)])
    if args.record is None:parser.error('An owned diagnostic record is required')
    gio=C.CDLL(ctypes.util.find_library('gio-2.0'))
    glib=C.CDLL(ctypes.util.find_library('glib-2.0'))
    objects=C.CDLL(ctypes.util.find_library('gobject-2.0'))
    def bind(library,name,result,*arguments):
        fn=getattr(library,name);fn.restype=result;fn.argtypes=list(arguments);return fn
    pointer=C.c_void_p;string=C.c_char_p;integer=C.c_int
    bus=bind(gio,'g_bus_get_sync',pointer,integer,pointer,pointer)
    node_new=bind(gio,'g_dbus_node_info_new_for_xml',pointer,string,pointer)
    lookup=bind(gio,'g_dbus_node_info_lookup_interface',pointer,pointer,string)
    register=bind(gio,'g_dbus_connection_register_object',C.c_uint,pointer,string,pointer,pointer,pointer,pointer,pointer)
    unregister=bind(gio,'g_dbus_connection_unregister_object',integer,pointer,C.c_uint)
    close_connection=bind(gio,'g_dbus_connection_close_sync',integer,pointer,pointer,pointer)
    call=bind(gio,'g_dbus_connection_call_sync',pointer,pointer,string,string,string,string,pointer,pointer,integer,integer,pointer,pointer)
    error_reply=bind(gio,'g_dbus_method_invocation_return_dbus_error',None,pointer,string,string)
    empty_reply=bind(gio,'g_dbus_method_invocation_return_value',None,pointer,pointer)
    parse=bind(glib,'g_variant_parse',pointer,pointer,string,pointer,pointer,pointer)
    sink=bind(glib,'g_variant_ref_sink',pointer,pointer)
    unref=bind(glib,'g_variant_unref',None,pointer)
    child_value=bind(glib,'g_variant_get_child_value',pointer,pointer,C.c_size_t)
    get_string=bind(glib,'g_variant_get_string',string,pointer,pointer)
    free_error=bind(glib,'g_error_free',None,pointer)
    class Error(C.Structure):
        _fields_=[('domain',C.c_uint),('code',integer),('message',string)]
    iterate=bind(glib,'g_main_context_iteration',integer,pointer,integer)
    node_unref=bind(gio,'g_dbus_node_info_unref',None,pointer)
    object_unref=bind(objects,'g_object_unref',None,pointer)
    method_type=C.CFUNCTYPE(None,pointer,string,string,string,string,pointer,pointer,pointer)
    class VTable(C.Structure):
        _fields_=[('method_call',method_type),('get_property',pointer),('set_property',pointer),('padding',pointer*8)]
    observed={'challenge_observed':False,'reply_error':'org.freedesktop.PolicyKit1.Error.Cancelled'}
    @method_type
    def method(connection,sender,path,interface,name,parameters,invocation,data):
        # GIO's public ABI owns the invocation until exactly one reply. The callback,
        # zero-initialized vtable and interface remain live until unregister below.
        if name==b'CancelAuthentication':
            empty_reply(invocation,None);return
        value=None
        try:
            if name!=b'BeginAuthentication':raise ValueError('Unexpected agent method')
            value=child_value(parameters,0)
            action=get_string(value,None)
            if action!=b'org.xdzip.update':raise ValueError('Unexpected authentication action')
            observed.update(challenge_observed=True,action=action.decode('ascii'))
            error_reply(invocation,b'org.freedesktop.PolicyKit1.Error.Cancelled',b'Owned fixture dismisses authentication')
        except Exception as error:
            observed['callback_error']=type(error).__name__
            error_reply(invocation,b'org.freedesktop.PolicyKit1.Error.Failed',b'Owned fixture rejects unexpected challenge')
        finally:
            if value:unref(value)
    table=VTable(method,None,None,(pointer*8)())
    read_fd,write_fd=os.pipe()
    process=None;connection=None;node=None;object_id=0;registered=False;subject=None
    def authority(name,text):
        value=parse(None,text.encode('ascii'),None,None,None)
        if not value:raise RuntimeError('Invalid fixed agent registration tuple')
        sink(value)
        error=pointer()
        try:
            reply=call(connection,b'org.freedesktop.PolicyKit1',b'/org/freedesktop/PolicyKit1/Authority',AUTHORITY,name,value,None,0,10000,None,C.byref(error))
            if not reply:
                detail=C.cast(error,C.POINTER(Error)).contents.message.decode('utf-8') if error else 'No error detail'
                raise RuntimeError('Authority rejected owned agent registration: '+detail)
            unref(reply)
        finally:
            if error:free_error(error)
            unref(value)
    try:
        process=subprocess.Popen([sys.executable,__file__,'--request',str(args.request),'--invoke-fd',str(read_fd)],pass_fds=(read_fd,))
        os.close(read_fd);read_fd=-1
        start=int(Path('/proc/'+str(process.pid)+'/stat').read_text().rsplit(')',1)[1].split()[19])
        subject="('unix-process', {'pid': <uint32 %d>, 'start-time': <uint64 %d>, 'uid': <int32 19731>})"%(process.pid,start)
        connection=bus(1,None,None)
        node=node_new(XML,None)
        if not connection or not node:raise RuntimeError('System GIO could not create the owned agent')
        object_id=register(connection,PATH,lookup(node,AGENT),C.byref(table),None,None,None)
        if not object_id:raise RuntimeError('Owned agent object registration failed')
        authority(b'RegisterAuthenticationAgent',"("+subject+", 'C.UTF-8', '"+PATH.decode()+"')")
        registered=True
        os.write(write_fd,b'1');os.close(write_fd);write_fd=-1
        deadline=time.monotonic()+30
        while process.poll() is None and time.monotonic()<deadline:
            iterate(None,False);time.sleep(.01)
        observed['helper_exit']=process.wait(timeout=1)
        if not observed['challenge_observed'] or observed.get('callback_error'):
            raise RuntimeError('Genuine expected authentication challenge was not dismissed')
    finally:
        for fd in (read_fd,write_fd):
            if fd>=0:os.close(fd)
        if process and process.poll() is None:
            process.terminate();process.wait(timeout=5)
        if registered:
            try:authority(b'UnregisterAuthenticationAgent',"("+subject+", '"+PATH.decode()+"')")
            except RuntimeError:
                # The authenticated subject has exited. Closing this owned bus
                # connection also revokes its agent, including an expired registration.
                observed['explicit_unregister_returned_error']=True
        if object_id:unregister(connection,object_id)
        if node:node_unref(node)
        if connection:
            observed['agent_connection_closed']=bool(close_connection(connection,None,None))
            object_unref(connection)
        args.record.write_text(json.dumps(observed,indent=2)+'\n')
        if connection and not observed['agent_connection_closed']:
            raise RuntimeError('Owned authentication-agent connection did not close')


if __name__=='__main__':main()
