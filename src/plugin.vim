" Initialize the channel
if !exists('s:calculatorJobId')
	let s:calculatorJobId = 0
endif

" The path to the binary that was created out of 'cargo build' or 'cargo build --release". This will generally be 'target/release/name'
let s:path = expand('<sfile>:p:h')
let s:bin = s:path . '/nvim-markers'
" echo s:path
" echo s:bin


" Entry point. Initialize RPC. If it succeeds, then attach commands to the `rpcnotify` invocations.
function! s:connect()
  let id = s:initRpc()
  
  if 0 == id
    echoerr "calculator: cannot start rpc process"
  elseif -1 == id
    echoerr "calculator: rpc process is not executable"
  else
    " Mutate our jobId variable to hold the channel ID
    let s:calculatorJobId = id 
    
    " call s:configureCommands() 
    command! -nargs=0 Push           :call rpcnotify(s:calculatorJobId, "push")
    command! -nargs=0 Pop            :call rpcnotify(s:calculatorJobId, "pop")
    command! -nargs=0 DisplayMarkers :call rpcnotify(s:calculatorJobId, "display")
  endif
endfunction

" Initialize RPC
function! s:initRpc()
  if s:calculatorJobId <= 0
    let jobid = jobstart([s:bin], { 'rpc': v:true })
    return jobid
  else
    return s:calculatorJobId
  endif
endfunction

" function! s:configureCommands()
"   command! -nargs=+ Add :call s:add(<f-args>)
"   command! -nargs=+ Multiply :call s:multiply(<f-args>)
" endfunction
"
" " Constants for RPC messages.
" let s:Add = 'add'
" let s:Multiply = 'multiply'
"
" function! s:add(...)
"   let s:p = get(a:, 1, 0)
"   let s:q = get(a:, 2, 0)
"
"   call rpcnotify(s:calculatorJobId, s:Add, str2nr(s:p), str2nr(s:q))
" endfunction
"
" function! s:multiply(...)
"   let s:p = get(a:, 1, 1)
"   let s:q = get(a:, 2, 1)
"
"   call rpcnotify(s:calculatorJobId, s:Multiply, str2nr(s:p), str2nr(s:q))
" endfunction

call s:connect()
