import base64
import hashlib
import os
import qrcode
import re
import sys
import tkinter as Tk
import tkinter.filedialog, tkinter.messagebox
import tempfile
from PIL import ImageTk
import zipfile

def makeSimpleQR( str, err_cor ) :
    qr = qrcode.QRCode( error_correction = err_cor, box_size = 2, border = 8)
    qr.add_data( str )
    qr.make()
    im = qr.make_image( fill_color = 'black', back_color = 'white' )
    return [ ImageTk.PhotoImage( im ) ]

def outputQR( ix, qrHead, b64, fm, to, err_cor ) :
    print( '[ {}, {} ]'.format( fm, to ) )
    qr = qrcode.QRCode( error_correction = err_cor, box_size = 2, border = 8)
    qr.add_data( qrHead + b64[ fm : to ] )
    qr.make()
    im = qr.make_image( fill_color = 'black', back_color = 'white' )
    return ImageTk.PhotoImage( im )

errCorrTab = [
    ( qrcode.constants.ERROR_CORRECT_L, ( 2953, 'L (7%) 2,953 byte' ) ),
    ( qrcode.constants.ERROR_CORRECT_M,
                                ( 2331, 'M (15%, default) 2,331 byte' ) ),
    ( qrcode.constants.ERROR_CORRECT_Q, ( 1663, 'Q (25%) 1,663 byte' ) ),
    ( qrcode.constants.ERROR_CORRECT_H, ( 1272, 'H (30%) 1,272 byte' ) ) ]
def makeQR( ifn, err_cor ) :
    global errCorrTab
    qrc = []
    with open( ifn, 'rb' ) as f :
        a = f.read()
        b64 = base64.b64encode( a ).decode( 'utf-8' )
        print( 'size = {}'.format( len( b64 ) ) )
        csiz = None
        for k, (s, _) in errCorrTab :
            if k == err_cor :
                csiz = s
                break
        basename = os.path.basename( ifn )
        csiz -= len( 'abcd:01:10:{}:'.format( basename ) )
        qrHash = hashlib.sha256( ( b64 + '{}'.format(err_cor) ).encode() ) \
            .hexdigest()[0:4]
        last = ( len( b64 ) + csiz - 1 ) // csiz
        qrHeadFmt = qrHash + ':{:02}:' + '{:02}:{}:'.format( last, basename )

        for i in range( 0, len( b64 ) - csiz + 1, csiz ) :
            qrHead = qrHeadFmt.format( i // csiz )
            q = outputQR( i // csiz, qrHead, b64, i, i + csiz, err_cor )
            qrc.append( q )
        if len( b64 ) % csiz != 0 :
            ix = len( b64 ) // csiz
            qrHead = qrHeadFmt.format( last - 1 )
            q = outputQR( ix, qrHead, b64, ix * csiz, len( b64 ), err_cor )
            qrc.append( q )
    return qrc

reg_qr = re.compile(
        r'([\da-f][\da-f][\da-f][\da-f]):(\d\d):(\d\d):([^:]+):([+/\w]+=*)' )
def mergeBase64( ifn ) :
    global reg_qr
    with open( ifn, 'r' ) as f :
        hsh = None
        tl = None
        ofn = None
        cts = {}
        for line in f :
            line = re.sub( r'\r?\n$', '', line )
            m = reg_qr.search( line )
            if m == None :
                print( 'skip : ' + line )
                continue
            if hsh == None :
                hsh = m.group( 1 )
                tl = m.group( 3 )
                ofn = m.group( 4 )
            elif hsh != m.group( 1 ) or tl != m.group( 3 ) \
                                    or ofn != m.group( 4 ) :
                continue
            cts[ str( int( m.group( 2 ) ) ) ] = m.group( 5 )
        sum = ''
        for i in range( int( tl ) ) :
            if not str( i ) in cts :
                sum = ''
                print( 'Detects lack parts : {}'.format( i ) )
                break
            sum += cts[ str( i ) ]
        if sum != '' :
            dir = os.path.dirname( ifn )
            with open( os.path.join( dir, ofn ), 'wb' ) as of :
                of.write( base64.b64decode( sum ) )

btn_fn = None
txt_fn = None
bln_zip = None
chk_zip = None
opt_err_var = None
str_inMethod = None
txt_direct = None
btn_dec = None
btn_head = None
btn_next = None
img = None
img_no = 0
canvas = None

def file_btn_click() :
    global txt_fn
    fTyp = [ ('', '*') ]
    iDir = os.path.abspath( os.path.dirname( __file__ ) )
    ifn = Tk.filedialog.askopenfilename(
                                filetypes = fTyp, initialdir = iDir)
    txt_fn.delete( 0, Tk.END )
    txt_fn.insert( Tk.END, ifn )

def disp_qr() :
    global canvas
    global img
    global img_no
    global txt_qrno
    global btn_head
    global btn_next
    canvas.create_image( 0, 0, image = img[ img_no ], anchor= Tk.NW )
    txt_qrno.set( '{} / 0 - {}'.format( img_no, len( img ) - 1 ) )
    if img_no == 0 :
        btn_head.configure( state = 'disabled' )
    else :
        btn_head.configure( state = 'normal' )
    if img_no + 1 == len( img ) :
        btn_next.configure( state = 'disabled' )
    else :
        btn_next.configure( state = 'normal' )

def next_btn_click() :
    global img
    global img_no
    if img_no + 1 != len( img ) :
        img_no += 1
    disp_qr()

def head_btn_click() :
    global img_no
    img_no = 0
    disp_qr()

def qrcode_btn_click() :
    global txt_fn
    global bln_zip
    global opt_err_var
    global errCorrTab
    global canvas
    global txt_qrno
    global str_inMethod
    global txt_direct
    global btn_head
    global btn_next
    global img

    val = None
    for ( v, ( s, d ) ) in errCorrTab :
        if opt_err_var.get() == d :
            val = v
            break
    img = []
    if str_inMethod.get() == 'text' :
        img = makeSimpleQR( txt_direct.get(), val )
    elif not bln_zip.get() :
        img = makeQR( txt_fn.get(), val )
    else :
        with tempfile.TemporaryDirectory() as tmpDn :
            tmpFn = os.path.join( tmpDn, 'temp.zip' )
            with zipfile.ZipFile( tmpFn, 'w', zipfile.ZIP_DEFLATED ) as zipF :
                fn = os.path.basename( txt_fn.get() )
                zipF.write( txt_fn.get(), fn )
            img = makeQR( tmpFn, val )

    qrWin = Tk.Toplevel()
    qrWin.geometry( '385x425' )
    qrWin.title('QR code')
    btn_head = Tk.Button( qrWin, text='<<', command= head_btn_click )
    btn_head.place( x=5, y=5 )

    txt_qrno = Tk.StringVar()
    txt_qrno.set( '' )
    lbl_qrno = Tk.Label( qrWin, textvariable= txt_qrno )
    lbl_qrno.place( x=100, y=5 )

    btn_next = Tk.Button( qrWin, text='>', command= next_btn_click )
    btn_next.place( x=190, y=5 )
    canvas = Tk.Canvas( qrWin, bg = 'white', width= 385, height= 385 )
    canvas.place( x = 0, y = 40 )
    head_btn_click()
    qrWin.mainloop()

def decode_btn_click() :
    global txt_fn
    mergeBase64( txt_fn.get() )

def inMethChange() :
    global str_inMethod
    global btn_fn
    global txt_fn
    global chk_zip
    global btn_dec
    global txt_direct
    if str_inMethod.get() == 'text' :
        btn_fn.configure( state = 'disabled' )
        txt_fn.configure( state = 'disabled' )
        chk_zip.configure( state = 'disabled' )
        btn_dec.configure( state = 'disabled' )
        txt_direct.configure( state = 'normal' )
    else :
        btn_fn.configure( state = 'normal' )
        txt_fn.configure( state = 'normal' )
        chk_zip.configure( state = 'normal' )
        btn_dec.configure( state = 'normal' )
        txt_direct.configure( state = 'disabled' )

def gui() :
    global btn_fn
    global txt_fn
    global bln_zip
    global chk_zip
    global opt_err_var
    global errCorrTab
    global str_inMethod
    global txt_direct
    global btn_dec

    root = Tk.Tk()
    root.geometry( '420x200' )
    root.title('Any File to QRcodes ')

    frm_base = Tk.Frame( root, relief = 'flat' )
    frm_base.pack()

    frm_in = Tk.LabelFrame( frm_base, text = 'Input' )
    frm_in.pack( fill = Tk.X )
    frm_out = Tk.LabelFrame( frm_base, text = 'Output' )
    frm_out.pack( side = Tk.LEFT )
    frm_out_qr = Tk.LabelFrame( frm_out, text = 'QR Code' )
    frm_out_qr.pack( side = Tk.LEFT )
    frm_out_dec = Tk.LabelFrame( frm_out, text = 'Decode base64' )
    frm_out_dec.pack( side = Tk.LEFT )

    frm_in_file = Tk.Frame( frm_in, relief = 'flat' )
    frm_in_file.pack( side = Tk.TOP )
    frm_in_zip = Tk.Frame( frm_in, relief = 'flat' )
    frm_in_zip.pack( side = Tk.TOP )
    frm_in_txt = Tk.Frame( frm_in, relief = 'flat' )
    frm_in_txt.pack( side = Tk.TOP )

    str_inMethod = Tk.StringVar()
    str_inMethod.set( 'file' )

    rad_fn = Tk.Radiobutton( frm_in_file, text='File Name',
            variable = str_inMethod, value = 'file', command = inMethChange )
    rad_fn.pack( side = Tk.LEFT )
    txt_fn = Tk.Entry( frm_in_file, width = 40 )
    txt_fn.pack( side = Tk.LEFT )
    btn_fn = Tk.Button( frm_in_file, text='Open...', command= file_btn_click )
    btn_fn.pack( side = Tk.LEFT )

    bln_zip = Tk.BooleanVar()
    bln_zip.set( False )
    chk_zip = Tk.Checkbutton( frm_in_zip, variable= bln_zip,
                                    text='with ZIP compression' )
    chk_zip.pack( side = Tk.LEFT )

    rad_direct = Tk.Radiobutton( frm_in_txt, text='Direct Text',
            variable = str_inMethod, value = 'text', command = inMethChange )
    rad_direct.pack( side = Tk.LEFT )
    txt_direct = Tk.Entry( frm_in_txt, width = 48 )
    txt_direct.pack( side = Tk.LEFT )

    frm_err = Tk.Frame( frm_out_qr, relief = 'flat' )
    frm_err.pack( side = Tk.TOP )
    lbl_fm = Tk.Label( frm_err, text='Error Correct' )
    lbl_fm.pack( side = Tk.LEFT )
    opt_err_var = Tk.StringVar( root )
    OptionList = []
    for _, attr in errCorrTab :
        _, desc = attr
        OptionList.append( desc )
    opt_err_var.set( OptionList[ 0 ] )
    opt_err = Tk.OptionMenu( frm_err, opt_err_var, *OptionList )
    opt_err.config( width= 25 )
    opt_err.pack()
    opt_err.pack( side = Tk.LEFT )

    btn = Tk.Button( frm_out_qr,
                text='Display QR codes', command= qrcode_btn_click )
    btn.pack( side = Tk.TOP )

    btn_dec = Tk.Button( frm_out_dec,
                text='Output Decoded file', command= decode_btn_click )
    btn_dec.pack( side = Tk.LEFT )

    inMethChange()
    root.mainloop()

if __name__ == '__main__' :
    gui()
