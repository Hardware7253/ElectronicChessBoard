; ModuleID = 'probe6.564ff6d072b6286e-cgu.0'
source_filename = "probe6.564ff6d072b6286e-cgu.0"
target datalayout = "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64"
target triple = "thumbv7m-none-unknown-eabi"

@alloc_49cd4770cae46c688efee7fa9c056542 = private unnamed_addr constant <{ [75 x i8] }> <{ [75 x i8] c"/rustc/8ede3aae28fe6e4d52b38157d7bfe0d3bceef225/library/core/src/num/mod.rs" }>, align 1
@alloc_275563c045e9c548af649a389b3d2136 = private unnamed_addr constant <{ ptr, [12 x i8] }> <{ ptr @alloc_49cd4770cae46c688efee7fa9c056542, [12 x i8] c"K\00\00\00~\04\00\00\05\00\00\00" }>, align 4
@str.0 = internal constant [25 x i8] c"attempt to divide by zero"

; probe6::probe
; Function Attrs: nounwind
define dso_local void @_ZN6probe65probe17h9742d3e1d4d112bcE() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false)
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h4fdce4c351e1a8dcE.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17h9430a05cd9c7a431E(ptr align 1 @str.0, i32 25, ptr align 4 @alloc_275563c045e9c548af649a389b3d2136) #3
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h4fdce4c351e1a8dcE.exit": ; preds = %start
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(none)
declare i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn nounwind
declare dso_local void @_ZN4core9panicking5panic17h9430a05cd9c7a431E(ptr align 1, i32, ptr align 4) unnamed_addr #2

attributes #0 = { nounwind "frame-pointer"="all" "target-cpu"="generic" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(none) }
attributes #2 = { cold noinline noreturn nounwind "frame-pointer"="all" "target-cpu"="generic" }
attributes #3 = { noreturn nounwind }
