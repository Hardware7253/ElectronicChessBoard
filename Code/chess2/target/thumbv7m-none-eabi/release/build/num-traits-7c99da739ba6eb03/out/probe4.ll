; ModuleID = 'probe4.d3d0aba72ecba11c-cgu.0'
source_filename = "probe4.d3d0aba72ecba11c-cgu.0"
target datalayout = "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64"
target triple = "thumbv7m-none-unknown-eabi"

; probe4::probe
; Function Attrs: nounwind
define dso_local void @_ZN6probe45probe17h2c8d2eec1da71303E() unnamed_addr #0 {
start:
  %0 = alloca i32, align 4
  store i32 1, ptr %0, align 4
  %1 = load i32, ptr %0, align 4, !noundef !0
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i32 @llvm.cttz.i32(i32, i1 immarg) #1

attributes #0 = { nounwind "frame-pointer"="all" "target-cpu"="generic" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!0 = !{}
