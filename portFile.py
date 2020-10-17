
import base64
import hashlib
import os
import qrcode
import re
import sys
import tkinter, tkinter.filedialog, tkinter.messagebox
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

txt_fn = None
bln_zip = None
opt_err_var = None
img = None
img_next = 0
canvas = None
lbl_qrno = None
bln_direct = None
chk_direct = None
txt_direct = None

def file_btn_click() :
    global txt_fn
    fTyp = [ ('', '*') ]
    iDir = os.path.abspath( os.path.dirname( __file__ ) )
    ifn = tkinter.filedialog.askopenfilename(
                                filetypes = fTyp, initialdir = iDir)
    txt_fn.delete( 0, tkinter.END )
    txt_fn.insert( tkinter.END, ifn )

def next_btn_click() :
    global canvas
    global img
    global img_next
    global txt_qrno
    if img_next >= len( img ) :
        print( 'img_next error' )
        return
    canvas.create_image( 0, 0, image = img[ img_next ], anchor= tkinter.NW )
    txt_qrno.set( '{} / 0 - {}'.format( img_next, len( img ) - 1 ) )
    img_next += 1

def head_btn_click() :
    global img_next
    img_next = 0
    next_btn_click()

def qrcode_btn_click() :
    global txt_fn
    global bln_zip
    global opt_err_var
    global errCorrTab
    global canvas
    global txt_qrno
    global bln_direct
    global txt_direct
    global img

    val = None
    for ( v, ( s, d ) ) in errCorrTab :
        if opt_err_var.get() == d :
            val = v
            break
    img = []
    if bln_direct.get() :
        img = makeSimpleQR( txt_direct.get(), val )
    elif not bln_zip.get() :
        img = makeQR( txt_fn.get(), val )
    else :
        with tempfile.TemporaryDirectory() as tmpDn :
            tmpFn = tmpDn + '/' + 'temp.zip'
            with zipfile.ZipFile( tmpFn, 'w',
                                compression= zipfile.ZIP_DEFLATED ) as zipF :
                zipF.write( txt_fn.get(), txt_fn.get() )
                img = makeQR( tmpFn, val )

    qrWin = tkinter.Toplevel()
    qrWin.geometry( '385x425' )
    qrWin.title('QR code')
    btn_head = tkinter.Button( qrWin, text='<<', command= head_btn_click )
    btn_head.place( x=5, y=5 )

    txt_qrno = tkinter.StringVar()
    txt_qrno.set( '' )
    lbl_qrno = tkinter.Label( qrWin, textvariable= txt_qrno )
    lbl_qrno.place( x=100, y=5 )

    btn_next = tkinter.Button( qrWin, text='>', command= next_btn_click )
    btn_next.place( x=190, y=5 )
    canvas = tkinter.Canvas( qrWin, bg = 'white', width= 385, height= 385 )
    canvas.place( x = 0, y = 40 )
    head_btn_click()
    qrWin.mainloop()

def decode_btn_click() :
    global txt_fn
    mergeBase64( txt_fn.get() )

def gui() :
    global txt_fn
    global bln_zip
    global chk_zip
    global opt_err_var
    global errCorrTab
    global bln_direct
    global chk_direct
    global txt_direct

    root = tkinter.Tk()
    root.geometry( '300x200' )
    root.title('Any File to QRcodes ')

    lbl_fn = tkinter.Label( text='Input File Name' )
    lbl_fn.place( x=10, y=10 )
    txt_fn = tkinter.Entry( width=20 )
    txt_fn.place( x=100, y=10 )
    btn_fn = tkinter.Button( root, text='File name', command= file_btn_click )
    btn_fn.place( x=230, y=10 )

    bln_zip = tkinter.BooleanVar()
    bln_zip.set( False )
    chk_zip = tkinter.Checkbutton( root, variable= bln_zip,
                                    text='input after ZIP compression' )
    chk_zip.place( x=100, y=40 )

    bln_direct = tkinter.BooleanVar()
    bln_direct.set( False )
    chk_direct = tkinter.Checkbutton( root, variable= bln_direct,
                                    text='Input Direct Text' )
    chk_direct.place( x=10, y=70 )
    txt_direct = tkinter.Entry( width=27 )
    txt_direct.place( x=125, y=70 )

    lbl_fm = tkinter.Label( text='Error Correct' )
    lbl_fm.place( x=25, y=100 )
    opt_err_var = tkinter.StringVar( root )
    OptionList = []
    for _, attr in errCorrTab :
        _, desc = attr
        OptionList.append( desc )
    opt_err_var.set( OptionList[ 0 ] )
    opt_err = tkinter.OptionMenu( root, opt_err_var, *OptionList )
    opt_err.config( width= 25 )
    opt_err.pack()
    opt_err.place( x=100, y= 100 )

    btn = tkinter.Button( root,
                text='Display QR codes', command= qrcode_btn_click )
    btn.place( x=15, y=170 )

    btn_dc = tkinter.Button( root,
                text='Decode base64 file', command= decode_btn_click )
    btn_dc.place( x=180, y=170 )

    root.bind( '<Return>', lambda event: btn_click() )

    root.mainloop()

gui()

