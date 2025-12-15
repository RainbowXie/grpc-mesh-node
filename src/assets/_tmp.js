(function(){function r(e,n,t){function o(i,f){if(!n[i]){if(!e[i]){var c="function"==typeof require&&require;if(!f&&c)return c(i,!0);if(u)return u(i,!0);var a=new Error("Cannot find module '"+i+"'");throw a.code="MODULE_NOT_FOUND",a}var p=n[i]={exports:{}};e[i][0].call(p.exports,function(r){var n=e[i][1][r];return o(n||r)},p,p.exports,r,e,n,t)}return n[i].exports}for(var u="function"==typeof require&&require,i=0;i<t.length;i++)o(t[i]);return o}return r})()({1:[function(require,module,exports){
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hook_intent = hook_intent;
exports.printIntent = printIntent;
exports.gzipDecode = gzipDecode;
exports.getRealClass = getRealClass;
exports.callMethod = callMethod;
exports.getFieldValue = getFieldValue;
exports.printMethods = printMethods;
exports.get_stack = get_stack;
exports.printField = printField;
exports.getClassNameByjclass = getClassNameByjclass;
exports.hook_registerNativeV1 = hook_registerNativeV1;
exports.utf8ByteToUnicodeStr = utf8ByteToUnicodeStr;
exports.uuid = uuid;
exports.stringToByte = stringToByte;
exports.byteToString = byteToString;
exports.byteToHexString = byteToHexString;
exports.enableWebviewDebug = enableWebviewDebug;
exports.inflaterByteArray = inflaterByteArray;
exports.readInputStreamWithoutConsuming = readInputStreamWithoutConsuming;
exports.getJavaStackTraceString = getJavaStackTraceString;
exports.hook_callConstructors = hook_callConstructors;
exports.initializeNativeFunction = initializeNativeFunction;
exports.fopen = fopen;
exports.getStackTraceString = getStackTraceString;
exports.patch_exit = patch_exit;
exports.getApkFilePath = getApkFilePath;
exports.printSpecificAddrCpuContext = printSpecificAddrCpuContext;
exports.dumpMemoryRanges = dumpMemoryRanges;
exports.confirmAddress = confirmAddress;
exports.dumpThreadRegs = dumpThreadRegs;
exports.createApplicationContext = createApplicationContext;
exports.getApplicationContext = getApplicationContext;
exports.getPackageName = getPackageName;
exports.getDataPath = getDataPath;
exports.isPointerReadableWritable = isPointerReadableWritable;
exports.find_symbols_by_addr = find_symbols_by_addr;
exports.createFolder = createFolder;
exports.generateRandomString = generateRandomString;
exports.dumpSo = dumpSo;
exports.dumpJavaU8 = dumpJavaU8;
exports.jsArrayBufferToJavaByteArray = jsArrayBufferToJavaByteArray;
exports.javaByteToJsArrayBufferV2 = javaByteToJsArrayBufferV2;
exports.javaByteToJsArrayBuffer = javaByteToJsArrayBuffer;
exports.dumpMemory = dumpMemory;
exports.find_module_by_function_name = find_module_by_function_name;
exports.readMaps = readMaps;
exports.scanMemorySync = scanMemorySync;
exports.scanMemory = scanMemory;
exports.antiAnti = antiAnti;
exports.hook_LoadNativeLibrary = hook_LoadNativeLibrary;
exports.enumMaps = enumMaps;
exports.enumLists = enumLists;
let TAG = "[hookenv]";
var debug = true;
const jni_struct_array = {
    reserved0: 0,
    reserved1: 1,
    reserved2: 2,
    reserved3: 3,
    GetVersion: 4,
    DefineClass: 5,
    FindClass: 6,
    FromReflectedMethod: 7,
    FromReflectedField: 8,
    ToReflectedMethod: 9,
    GetSuperclass: 10,
    IsAssignableFrom: 11,
    ToReflectedField: 12,
    Throw: 13,
    ThrowNew: 14,
    ExceptionOccurred: 15,
    ExceptionDescribe: 16,
    ExceptionClear: 17,
    FatalError: 18,
    PushLocalFrame: 19,
    PopLocalFrame: 20,
    NewGlobalRef: 21,
    DeleteGlobalRef: 22,
    DeleteLocalRef: 23,
    IsSameObject: 24,
    NewLocalRef: 25,
    EnsureLocalCapacity: 26,
    AllocObject: 27,
    NewObject: 28,
    NewObjectV: 29,
    NewObjectA: 30,
    GetObjectClass: 31,
    IsInstanceOf: 32,
    GetMethodID: 33,
    CallObjectMethod: 34,
    CallObjectMethodV: 35,
    CallObjectMethodA: 36,
    CallBooleanMethod: 37,
    CallBooleanMethodV: 38,
    CallBooleanMethodA: 39,
    CallByteMethod: 40,
    CallByteMethodV: 41,
    CallByteMethodA: 42,
    CallCharMethod: 43,
    CallCharMethodV: 44,
    CallCharMethodA: 45,
    CallShortMethod: 46,
    CallShortMethodV: 47,
    CallShortMethodA: 48,
    CallIntMethod: 49,
    CallIntMethodV: 50,
    CallIntMethodA: 51,
    CallLongMethod: 52,
    CallLongMethodV: 53,
    CallLongMethodA: 54,
    CallFloatMethod: 55,
    CallFloatMethodV: 56,
    CallFloatMethodA: 57,
    CallDoubleMethod: 58,
    CallDoubleMethodV: 59,
    CallDoubleMethodA: 60,
    CallVoidMethod: 61,
    CallVoidMethodV: 62,
    CallVoidMethodA: 63,
    CallNonvirtualObjectMethod: 64,
    CallNonvirtualObjectMethodV: 65,
    CallNonvirtualObjectMethodA: 66,
    CallNonvirtualBooleanMethod: 67,
    CallNonvirtualBooleanMethodV: 68,
    CallNonvirtualBooleanMethodA: 69,
    CallNonvirtualByteMethod: 70,
    CallNonvirtualByteMethodV: 71,
    CallNonvirtualByteMethodA: 72,
    CallNonvirtualCharMethod: 73,
    CallNonvirtualCharMethodV: 74,
    CallNonvirtualCharMethodA: 75,
    CallNonvirtualShortMethod: 76,
    CallNonvirtualShortMethodV: 77,
    CallNonvirtualShortMethodA: 78,
    CallNonvirtualIntMethod: 79,
    CallNonvirtualIntMethodV: 80,
    CallNonvirtualIntMethodA: 81,
    CallNonvirtualLongMethod: 82,
    CallNonvirtualLongMethodV: 83,
    CallNonvirtualLongMethodA: 84,
    CallNonvirtualFloatMethod: 85,
    CallNonvirtualFloatMethodV: 86,
    CallNonvirtualFloatMethodA: 87,
    CallNonvirtualDoubleMethod: 88,
    CallNonvirtualDoubleMethodV: 89,
    CallNonvirtualDoubleMethodA: 90,
    CallNonvirtualVoidMethod: 91,
    CallNonvirtualVoidMethodV: 92,
    CallNonvirtualVoidMethodA: 93,
    GetFieldID: 94,
    GetObjectField: 95,
    GetBooleanField: 96,
    GetByteField: 97,
    GetCharField: 98,
    GetShortField: 99,
    GetIntField: 100,
    GetLongField: 101,
    GetFloatField: 102,
    GetDoubleField: 103,
    SetObjectField: 104,
    SetBooleanField: 105,
    SetByteField: 106,
    SetCharField: 107,
    SetShortField: 108,
    SetIntField: 109,
    SetLongField: 110,
    SetFloatField: 111,
    SetDoubleField: 112,
    GetStaticMethodID: 113,
    CallStaticObjectMethod: 114,
    CallStaticObjectMethodV: 115,
    CallStaticObjectMethodA: 116,
    CallStaticBooleanMethod: 117,
    CallStaticBooleanMethodV: 118,
    CallStaticBooleanMethodA: 119,
    CallStaticByteMethod: 120,
    CallStaticByteMethodV: 121,
    CallStaticByteMethodA: 122,
    CallStaticCharMethod: 123,
    CallStaticCharMethodV: 124,
    CallStaticCharMethodA: 125,
    CallStaticShortMethod: 126,
    CallStaticShortMethodV: 127,
    CallStaticShortMethodA: 128,
    CallStaticIntMethod: 129,
    CallStaticIntMethodV: 130,
    CallStaticIntMethodA: 131,
    CallStaticLongMethod: 132,
    CallStaticLongMethodV: 133,
    CallStaticLongMethodA: 134,
    CallStaticFloatMethod: 135,
    CallStaticFloatMethodV: 136,
    CallStaticFloatMethodA: 137,
    CallStaticDoubleMethod: 138,
    CallStaticDoubleMethodV: 139,
    CallStaticDoubleMethodA: 140,
    CallStaticVoidMethod: 141,
    CallStaticVoidMethodV: 142,
    CallStaticVoidMethodA: 143,
    GetStaticFieldID: 144,
    GetStaticObjectField: 145,
    GetStaticBooleanField: 146,
    GetStaticByteField: 147,
    GetStaticCharField: 148,
    GetStaticShortField: 149,
    GetStaticIntField: 150,
    GetStaticLongField: 151,
    GetStaticFloatField: 152,
    GetStaticDoubleField: 153,
    SetStaticObjectField: 154,
    SetStaticBooleanField: 155,
    SetStaticByteField: 156,
    SetStaticCharField: 157,
    SetStaticShortField: 158,
    SetStaticIntField: 159,
    SetStaticLongField: 160,
    SetStaticFloatField: 161,
    SetStaticDoubleField: 162,
    NewString: 163,
    GetStringLength: 164,
    GetStringChars: 165,
    ReleaseStringChars: 166,
    NewStringUTF: 167,
    GetStringUTFLength: 168,
    GetStringUTFChars: 169,
    ReleaseStringUTFChars: 170,
    GetArrayLength: 171,
    NewObjectArray: 172,
    GetObjectArrayElement: 173,
    SetObjectArrayElement: 174,
    NewBooleanArray: 175,
    NewByteArray: 176,
    NewCharArray: 177,
    NewShortArray: 178,
    NewIntArray: 179,
    NewLongArray: 180,
    NewFloatArray: 181,
    NewDoubleArray: 182,
    GetBooleanArrayElements: 183,
    GetByteArrayElements: 184,
    GetCharArrayElements: 185,
    GetShortArrayElements: 186,
    GetIntArrayElements: 187,
    GetLongArrayElements: 188,
    GetFloatArrayElements: 189,
    GetDoubleArrayElements: 190,
    ReleaseBooleanArrayElements: 191,
    ReleaseByteArrayElements: 192,
    ReleaseCharArrayElements: 193,
    ReleaseShortArrayElements: 194,
    ReleaseIntArrayElements: 195,
    ReleaseLongArrayElements: 196,
    ReleaseFloatArrayElements: 197,
    ReleaseDoubleArrayElements: 198,
    GetBooleanArrayRegion: 199,
    GetByteArrayRegion: 200,
    GetCharArrayRegion: 201,
    GetShortArrayRegion: 202,
    GetIntArrayRegion: 203,
    GetLongArrayRegion: 204,
    GetFloatArrayRegion: 205,
    GetDoubleArrayRegion: 206,
    SetBooleanArrayRegion: 207,
    SetByteArrayRegion: 208,
    SetCharArrayRegion: 209,
    SetShortArrayRegion: 210,
    SetIntArrayRegion: 211,
    SetLongArrayRegion: 212,
    SetFloatArrayRegion: 213,
    SetDoubleArrayRegion: 214,
    RegisterNatives: 215,
    UnregisterNatives: 216,
    MonitorEnter: 217,
    MonitorExit: 218,
    GetJavaVM: 219,
    GetStringRegion: 220,
    GetStringUTFRegion: 221,
    GetPrimitiveArrayCritical: 222,
    ReleasePrimitiveArrayCritical: 223,
    GetStringCritical: 224,
    ReleaseStringCritical: 225,
    NewWeakGlobalRef: 226,
    DeleteWeakGlobalRef: 227,
    ExceptionCheck: 228,
    NewDirectByteBuffer: 229,
    GetDirectBufferAddress: 230,
    GetDirectBufferCapacity: 231,
    GetObjectRefType: 232,
};
let unsetWatchPoint = false;
let _addr, _size, _conditions;
let threads;
function installWatchpoint(addr, size, conditions) {
    _addr = addr;
    _size = size;
    _conditions = conditions;
    threads = Process.enumerateThreads();
    Process.setExceptionHandler((e) => {
        if (["breakpoint", "single-step"].includes(e.type)) {
            console.log(`\n[!] ${e.context.pc} tried to "${_conditions}" at ${_addr}`);
            for (const thread of threads) {
                if (thread.id === Process.getCurrentThreadId()) {
                    thread.unsetHardwareWatchpoint(0);
                    unsetWatchPoint = true;
                    return true;
                }
            }
        }
        return false;
    });
    for (const thread of threads) {
        try {
            thread.setHardwareWatchpoint(0, addr, size, conditions);
            console.log(`[*] HardwareWatchpoint set at ${addr} (${thread.id} ${thread.name})`);
        }
        catch (error) { }
    }
}
function reInstallWatchPoint() {
    for (const thread of threads) {
        try {
            thread.setHardwareWatchpoint(0, _addr, _size, _conditions);
        }
        catch (error) { }
    }
}
//自动识别map和hashmap，ArrayList
function show_object(obj) {
    // var result = "null";
    // if (typeof obj == 'object') {
    //     // 如果返回值是对象，将其转换为字符串
    //     var objString = JSON.stringify(obj);
    //     // console.log('arguments JSON.stringify(obj)==>: ', objString)
    //     // console.log('arguments obj==> ', obj)
    //     // console.log("obj.classtype==> " + obj.$className)  //java.util.Collections$UnmodifiableMap
    //     if (objString.indexOf('java.io.ByteArrayOutputStream') >= 0 ) {
    //         // show_map(obj)
    //         result = show_OutputStream(obj)
    //         // gson_show_hashMap(obj)
    //         if (result != "")
    //             return "show_OutputStream(obj):\n" + result
    //         return result
    //     }
    //     else if (objString.indexOf('[') >= 0) {
    //         var length = obj.length
    //         if (length > 0) {
    //             result = str2hxex(objString)
    //             if (result != "")
    //                 return "str2hxex(obj):\n " + result
    //         }
    //         return result
    //     }
    //     else if (objString.indexOf('java.util.List') >= 0) {
    //         var size = obj.size();
    //         console.log("List size: " + size);
    //         // 遍历 List 并打印每个元素
    //         for (var i = 0; i < size; i++) {
    //             var element = obj.get(i);
    //             console.log("Element " + i + ": " + element);
    //         }
    //     }
    //     else if (objString.indexOf('com.whatsapp.jid.DeviceJid') >= 0 ) {
    //         // show_map(obj)
    //         result = ""
    //         if (obj == null || obj == undefined)
    //             return "show_SharedPreferences(obj):\n" + result
    //         return obj.getRawString()
    //     }
    //     else if (objString.indexOf('android.content.SharedPreferences') >= 0 ) {
    //         // show_map(obj)
    //         result = show_SharedPreferences(obj)
    //         // gson_show_hashMap(obj)
    //         if (result != "")
    //             return "show_SharedPreferences(obj):\n" + result
    //         return result
    //     }
    //     // else if (objString.indexOf('X.9ur') >= 0) {
    //     //     result = str2hxex(JSON.stringify(obj.A02.value))
    //     //     // if (result != "")
    //     //     console.log("X.9ur.A02 " + result, "\nX.9ur.A00 " + obj.A00)
    //     //         // return "str2hxex(obj):\n X.9ur.A02 " + result + "\nX.9ur.A00 " + printArg(obj.A00)
    //     //     // return result
    //     //     return ""
    //     // }
    //     else if (objString.indexOf('X.1xI') >= 0) {
    //         console.log("X.1xI.A00", obj.A00)
    //         console.log("X.1xI.A01", obj.A01)
    //         // console.log("X.1xI.A02", obj.A02)
    //         console.log("X.1xI.A03", obj.A03)
    //         console.log("X.1xI.A04", obj.A04)
    //         console.log("X.1xI.A05", obj.A05)
    //         console.log("X.1xI.A06", obj.A06)
    //         console.log("X.1xI.A07", obj.A07)
    //         console.log("X.1xI.A08", obj.A08)
    //         console.log("X.1xI.A09", obj.A09)
    //         console.log("X.1xI.A0A", obj.A0A)
    //         console.log("X.1xI.A0B", obj.A0B)
    //         return ""
    //     }
    //     else if (objString.indexOf('java.util.LinkedHashMap') >= 0) {
    //         // show_map(obj)
    //         // result = show_LinkedHashMap(obj)
    //         gson_show_hashMap(obj)
    //         if (result != "")
    //             return "show_LinkedHashMap(obj):\n" + result
    //         return result
    //     }
    //     else if (objString.indexOf('java.util.HashMap') >= 0 ) {
    //         // show_map(obj)
    //         result = show_hashMap(obj)
    //         // gson_show_hashMap(obj)
    //         if (result != "")
    //             return "show_hashMap(obj):\n" + result
    //         return result
    //     }
    //                         //"Protobuf"                   "SyncdKey{keyId="            // 二维码                       // 电话参数
    //     else if (objString.indexOf('X.2FZ') >= 0 || objString.indexOf('X.2jR') || objString.indexOf('X.FeS')  >= 0 || objString.indexOf('X.1Sg')  >= 0||objString.indexOf('X.1NP')  >= 0 || objString.indexOf('X.1O5')  >= 0 ||objString.indexOf('X.1Rl')  >= 0 || objString.indexOf('X.1Uc')  >= 0 ||objString.indexOf('CallInfo')  >= 0)  {
    //         // show_map(obj)
    //         if (objString.indexOf('null') >= 0){
    //             return "null"
    //         }
    //         return obj.toString()
    //     }
    //     else if (objString.indexOf('java.lang.Integer') >= 0 ) {
    //         // show_map(obj)
    //         return obj.intValue()
    //     }
    //     else if (objString.indexOf('android.content.SharedPreferences') >= 0 ) {
    //         // show_map(obj)
    //         result = show_SharedPreferences(obj)
    //         // gson_show_hashMap(obj)
    //         if (result != "")
    //             return "show_SharedPreferences(obj):\n" + result
    //         return result
    //     }
    //     else if (objString.indexOf('<instance: android.os.Parcelable, $className:') >= 0) {
    //         // console.log("obj.classtype==> " + obj.$className)
    //         result = show_cast(obj, obj.$className)
    //         if (result != "")
    //             return "obj.toString():\n " + result
    //         return result
    //     }
    //     else if (objString.indexOf('className: java.lang.String') >= 0) {
    //         // console.log('objString Object==>: ', obj.toString())
    //         result = obj.toString()
    //         if (result != "")
    //             return "obj.toString():\n " + result
    //         return result
    //     }
    //     else if (objString.indexOf('java.util.Map' ||
    //         objString.indexOf('java.util.Collections$UnmodifiableMap') >= 0) >= 0 ||
    //         objString.indexOf('java.util.LinkedHashMap') >= 0) {
    //         result = show_map(obj)
    //         if (result != "")
    //             return "show_map(obj):\n " + result
    //         return result
    //     }
    //     else if (objString.indexOf('java.util.byte') >= 0) {
    //         result = show_arrb(obj)
    //         if (result != "")
    //             return "show_arrb(obj):\n " + result
    //         return result
    //     }
    //     else if (objString.indexOf('android.database.sqlite.SQLiteCursor') >= 0) {
    //         // result =
    //         // console.log("SQLiteCursor show_map(obj) ", show_map(obj))
    //         console.log(" SQLiteCursor show_map(obj) ", obj.toString())
    //         if (result != "")
    //             return "show_arrb(obj):\n " + result
    //         return result
    //     }
    //     else if (objString.indexOf('java.util.Arrays$ArrayList') >= 0 ||
    //         objString.indexOf('java.util.Collections$UnmodifiableRandomAccessList') >= 0 ||
    //         objString.indexOf('java.util.ArrayList') >= 0 ) {
    //         var arrays = Java.use('java.util.Arrays')
    //         result = arrays.toString(obj.toArray())
    //         // console.log('content: ' + result);
    //         if (result != "")
    //             return "show_ArrayList(obj):\n " + result
    //         return result
    //     }
    //     else if (objString.indexOf('java.security.Key') >= 0) {
    //         var arrays = Java.use('java.security.Key')
    //         // console.log('java.security.Key.getAlgorithm: ' + obj.getAlgorithm());
    //         // console.log('java.security.Key.getFormat: ' + obj.getFormat());
    //         // console.log('java.security.Key.getEncoded: ' + obj.getEncoded());
    //         // console.log('content: ' + result);
    //         return ""
    //     }
    //     else
    //     {
    //         console.log('obj ', JSON.stringify(obj))
    //     }
    // }
    // else
    // {
    //     console.log('obj ', JSON.stringify(obj))
    // }
}
//=========================== 获取参数类型，参数，OutputStream==================================
function show_OutputStream(output_stream_obj) {
    var result = "";
    var OutputStreamCls = Java.use("java.io.ByteArrayOutputStream");
    result = Java.cast(output_stream_obj, OutputStreamCls).toString();
    return result;
}
function hook_intent() {
    var intentCls = Java.use("android.content.Intent");
    // var testintent = true;
    intentCls["$init"].overload("java.lang.String").implementation = function (name) {
        let result = this["$init"](name);
        console.log(`Intent is called: str=${name}`);
        printIntent(this);
        console.log(`Intent result=${result}\n${getJavaStackTraceString()}`);
    };
    intentCls["$init"].overload("java.lang.String", "android.net.Uri").implementation = function (name, uri) {
        let result = this["$init"](name, uri);
        console.log(`Intent is called: str=${name}, uri=${uri}`);
        printIntent(this);
        console.log(`Intent result=${result}\n${getJavaStackTraceString()}`);
    };
    intentCls["$init"].overload("android.content.Intent").implementation = function (intent) {
        let result = this["$init"](intent);
        console.log(`Intent is called: intent=${intent}`);
        printIntent(this);
        console.log(`Intent result=${result}\n${getJavaStackTraceString()}`);
    };
    intentCls["$init"].overload("android.os.Parcel").implementation = function (parcel) {
        let result = this["$init"](parcel);
        console.log(`Intent is called: parcel=${parcel}`);
        printIntent(this);
        console.log(`Intent result=${result}\n${getJavaStackTraceString()}`);
    };
    intentCls["$init"].overload("android.content.Context", "java.lang.Class").implementation = function (context, cls) {
        let result = this["$init"](context, cls);
        console.log(`Intent is called: context=${context}, cls=${cls}`);
        printIntent(this);
        console.log(`Intent result=${result}\n${getJavaStackTraceString()}`);
    };
    intentCls["$init"].overload("android.content.Intent", "int").implementation = function (intent, flags) {
        let result = this["$init"](intent, flags);
        console.log(`Intent is called: intent=${intent}, flags=${flags}`);
        printIntent(this);
        console.log(`Intent result=${result}\n${getJavaStackTraceString()}`);
    };
    intentCls["$init"].overload("java.lang.String", "android.net.Uri", "android.content.Context", "java.lang.Class").implementation = function (str, uri, context, cls) {
        let result = this["$init"](str, uri, context, cls);
        console.log(`Intent is called: str=${str}, uri=${uri}, context=${context}, cls=${cls}`);
        printIntent(this);
        console.log(`Intent result=${result}\n${getJavaStackTraceString()}`);
    };
}
function printIntent(intent) {
    console.log(`Intent is called: intent=${JSON.stringify(intent)}`);
    if (intent != null) {
        // 获取 Action
        var action = intent.getAction();
        console.log("[*] Action: " + (action ? action : "null"));
        // 获取 Data
        var data = intent.getData();
        console.log("[*] Data: " + (data ? data.toString() : "null"));
        // 获取 Categories
        var categories = intent.getCategories();
        console.log("[*] Categories: " + (categories ? categories.toString() : "null"));
        // 获取 Extras
        var extras = intent.getExtras();
        if (extras) {
            var keys = extras.keySet();
            var iterator = keys.iterator();
            while (iterator.hasNext()) {
                var key = iterator.next();
                var value = extras.get(key);
                console.log("[*] Extra: " + key + " = " + value);
            }
        }
        else {
            console.log("[*] Extras: null");
        }
        // 获取 Component
        var component = intent.getComponent();
        console.log("[*] Component: " + (component ? component.toString() : "null"));
        // 获取 Flags
        var flags = intent.getFlags();
        console.log("[*] Flags: " + flags + " (0x" + flags.toString(16) + ")");
        // 获取 Type
        var type = intent.getType();
        console.log("[*] Type: " + (type ? type : "null"));
        // 获取 Package
        var packageName = intent.getPackage();
        console.log("[*] Package: " + (packageName ? packageName : "null"));
    }
}
// hook指定重载函数 应对重载混淆最佳
// export function hook_overloads_fun(cls_name:any, method_name:any) {
//     // console.log("-------------------------------- hook_overloads_fun() hook指定函数 --------------------------")
//     console.log("hook method_name ==> ", cls_name + "." + method_name)
//     var obj = Java.classFactory.use(cls_name)
//     if (obj != undefined) {
//         // console.log("obj ==> ", obj)
//         var method = obj[method_name];
//         if (!method) {
//             console.log("no mothod return ", method_name)
//             return;
//         }
//         // console.log("find===> ", m.getDeclaringClass().getName() + '.' + name)
//         var overloads = method.overloads;
//         console.log("所有的重载方法 ==>", overloads)
//         for (var overload of overloads) {
//             // console.log("[+] overload ==> ", overload)
//             // 打印参数类型 2
//             var proto = getArg_type(overload)
//             // console.log("[+] proto ==> ", proto)
//             //重载操作
//             // console.log("[+] hooking==> ", cls_name + '.' + method_name, getArg_type(overload))
//             overload.implementation = function () {
//                 var ret;
//                 if(method_name == "nOnResume")
//                 {
//                     console.log("[+] nOnResume Close")
//                     return
//                 }
//                 if(method_name == "run" && cls_name == "X.3QS"){
//                     var instance = this
//                     console.log("[+] X.3QS run", instance.A01)
//                     if(instance.A01 != null){
//                         // console.log("[+] X.3QS run", instance.A01.A02.value)
//                         var x7L4_calss = Java.use("X.7L4");
//                         var x7L4 = Java.cast(instance.A01, x7L4_calss)
//                         x7L4.A02.value = "123123123123"
//                     }
//                     // var keyBytes = secretKey.getEncoded();
//                 }
//                 if(cls_name == "java.lang.String" && method_name =="format"){
//                     if(arguments[0] == "%s,%s,%s,%s,%s")
//                         {
//                         // var securitykey = Java.use("java.security.Key");
//                         // var KeyGenerator = Java.use("javax.crypto.KeyGenerator");
//                         // var keyGen = KeyGenerator.getInstance("HmacSHA256");
//                         // var secretKey = keyGen.generateKey();
//                         // // console.log(secretKey.$className);
//                         // secretKey = Java.cast(secretKey, securitykey)
//                         // var keyBytes = secretKey.getEncoded();
//                         // 用下面的就行
//                         // const randomBytes = new Uint8Array(32);
//                         // for (let i = 0; i < randomBytes.length; i++) {
//                         //     randomBytes[i] = Math.floor(Math.random() * 256);
//                         // }
//                         // var Base64 = Java.use('android.util.Base64');
//                         // var base64Random = Base64.encodeToString(Java.array('byte',randomBytes), 2)
//                         // // 分割字符串并替换第四段
//                         // console.log("[Set QRCode]", arguments[1][3], "==>", base64Random)
//                         // arguments[1][3] = base64Random;
//                         console.log("[QRCode]", show_object(arguments[1]))
//                         return this[method_name].apply(this, arguments);
//                     }
//                     else{
//                         return this[method_name].apply(this, arguments);
//                     }
//                 }
//                 // if (cls_name == "X.0x0" && method_name =="A0b"){
//                 //     var objString = JSON.stringify(arguments[0])
//                 //     if (objString.indexOf("X.2jR") >= 0 ) {
//                 //         console.log("[SyncdKey]", show_object(arguments[0]))
//                 //         return this[method_name].apply(this, arguments);
//                 //     }
//                 //     else{
//                 //         return this[method_name].apply(this, arguments);
//                 //     }
//                 // }
//                 // Hook_Get_wa_key()
//                 //console.log("[+] 当前调用重载方法==> ", overload)
//                 // 打印调用方法
//                 console.log("[+] hooking==> ", cls_name + '.' + method_name + proto)
//                 // 打印参数 2
//                 var buff = ""
//                 var i = 0;
//                 console.log("    arguments.len: ", arguments.length)
//                 for (var arg of arguments) {
//                     // console.log("    Argv[" + i + "]", arg)
//                     // console.log("    Args className==>  ", typeof arg)
//                     if (arg !== null && arg !== undefined) {
//                         //console.log("[+] Args className==>  ", typeof arg)
//                         var value = ""
//                         value = printArg(arg)
//                         if (value != "") {
//                             buff += "    Argv[" + i + "] " + value + "\n"
//                         }
//                         else
//                             buff += "    Argv[" + i + "] " + arg.toString + "\n"
//                     }
//                     console.log("    Argv[" + i + "]", value)
//                     i++;
//                 }
//                 // 执行获取返回值
//                 ret = this[method_name].apply(this, arguments);
//                 // 获取返回值类型
//                 var returnType = typeof ret
//                 // console.log('    ' + cls_name + "[ " + overload + " ]" + ' Return Value Type:', returnType);
//                 // console.log('[+] ' + cls_name + "[ " + overload + " ]" + ' Return Value:', ret);
//                 // show_arrb(ret)
//                 // console.log("Return : ", bytes2Hex(obj))
//                 // console.log("    Return printArg(ret) : ", show_object(ret))
//                 // console.log("    Return : ", ret)
//                 if (ret !== null && ret !== undefined && returnType !== undefined) {
//                     // console.log("ret.classtype==> " + ret.$className)
//                    console.log("    Return (ret) : ", show_object(ret))
//                 }
//                 else{
//                     console.log("    Return : ", ret)
//                 }
//                 console.log("\n")
//                 return ret
//             }
//         }
//     }
//     // console.log("-------------------------------- hook_overloads_fun() end--------------------------")
// }
function gzipDecode(encodedByte) {
    var ByteArrayInputStreamCls = Java.use("java.io.ByteArrayInputStream");
    var byteArrayInputStream = ByteArrayInputStreamCls.$new(encodedByte);
    var GZIPInputStreamCls = Java.use("java.util.zip.GZIPInputStream");
    var gzipInputStream = GZIPInputStreamCls.$new(byteArrayInputStream);
    var byteArrayOutputStream = Java.use("java.io.ByteArrayOutputStream").$new();
    var arr = new Array(1024).fill(0);
    var bytes = Java.array("byte", arr);
    var readlen = 0;
    try {
        var inputStream = Java.cast(gzipInputStream, Java.use("java.io.InputStream"));
        while ((readlen = inputStream.read(bytes)) > 0) {
            byteArrayOutputStream.write(bytes, 0, readlen);
        }
    }
    catch (e) {
        console.log(TAG, e);
    }
    return byteArrayOutputStream.toByteArray();
}
function getRealClass(obj) {
    return Java.cast(obj, Java.use("java.lang.Object")).getClass();
}
function callMethod(obj, name) {
    var ret = null;
    var method = obj.getClass().getDeclaredMethod(name);
    method;
    return ret;
}
// @ts-ignore
function getFieldValue(obj, name) {
    var value = null;
    Java.perform(function () {
        var objClass = getRealClass(obj);
        for (; objClass !== null; objClass = objClass.getSuperclass()) {
            try {
                var field = objClass.getDeclaredField(name);
                if (field != null) {
                    field.setAccessible(true);
                    value = field.get(obj);
                    console.log("[Field Name]\t" + field.getName() + "\t[Field Value]\t" + field.get(obj));
                    break;
                }
            }
            catch (error) {
                // console.log("[-] [getFieldValue] " + error);
            }
        }
    });
    return value;
}
/**
 * 打赢对象所属类的所有方法
 * @param obj
 * @returns
 */
function printMethods(obj) {
    var strFuncs = [];
    Java.perform(function () {
        var objClass = getRealClass(obj);
        for (; objClass !== null; objClass = objClass.getSuperclass()) {
            var methods = objClass.getDeclaredMethods();
            for (var i = 0; i < methods.length; i++) {
                var method = methods[i];
                var modifierString = getModifierString(method.getModifiers());
                var methodName = method.getName();
                var params = method.getParameterTypes();
                var retType = method.getReturnType().getName(); // method.getGenericReturnType().getName();
                var paramTypes = params
                    .map(function (t) {
                    return t.getName();
                })
                    .join(", ");
                var methodString = modifierString + " " + retType + " " + methodName + "(" + paramTypes + ");";
                strFuncs.push(methodString);
            }
        }
    });
    return strFuncs;
}
/**
 * 使用 libxdl 和 libxunwind 打印堆栈
 *
 */
function get_stack(modpath) {
    var pid = Process.id;
    Module.load(modpath + "/libxdl.so");
    var md = Module.load(modpath + "/libxunwind.so");
    // var md = Module.load("/data/data/cn.com.njcb.android.mobilebank/files/libxunwind.so");
    var func_ptr = md.findExportByName("xunwind_cfi_get");
    // var open_ptr = Module.findExportByName("libc.so", "open");
    // var errno_ptr = Module.findExportByName("libc.so", "__errno")
    console.log("xunwind_cfi_get: " + func_ptr); // + " open: " + open_ptr + " __errno: " + errno_ptr);
    if (func_ptr) {
        //  && open_ptr && errno_ptr)
        var xunwind_cfi_get = new NativeFunction(func_ptr, "pointer", ["int", "int", "pointer", "pointer"]);
        // var open = new NativeFunction(open_ptr, 'int', ['pointer', 'int', 'uint']);
        // var errno = new NativeFunction(errno_ptr, 'pointer', [])
        // var filePath = "/data/data/cn.com.njcb.android.mobilebank/files/stack.log";
        // var pathptr = Memory.alloc(filePath.length + 1);
        // pathptr.writeUtf8String(filePath);
        // console.log(TAG, "save path: " + pathptr.readUtf8String())
        // const O_CREAT = 0x0100
        // const O_RDWR = 0x0002
        // const S_IRWXU = 0x00700
        // const S_IRWXG = 0x00070
        // const S_IRWXO = 0x00007
        // var fd = open(pathptr, O_RDWR | O_CREAT, S_IRWXU | S_IRWXG | S_IRWXO);
        // if (fd == -1)
        // {
        //     console.log("err: " + errno().readInt());
        // }
        // console.log("fd: ", fd);
        console.log("starting dump thread...");
        var buf = xunwind_cfi_get(pid, -2, ptr(0), ptr(0));
        console.log("dump thread over.");
        return buf;
    }
    return null;
}
function printField(obj) {
    var strFields = [];
    Java.perform(function () {
        var objClass = getRealClass(obj);
        for (; objClass !== null; objClass = objClass.getSuperclass()) {
            var fields = objClass.getDeclaredFields();
            for (var i = 0; i < fields.length; i++) {
                var field = fields[i];
                field.setAccessible(true);
                var modifierString = getModifierString(field.getModifiers());
                var fieldClass = field.getType().getName();
                var fieldName = field.getName();
                var fieldValue = field.get(obj);
                if (fieldValue != null) {
                    var fieldValueString = null;
                    try {
                        if (fieldClass == "java.util.List" || fieldClass == "java.util.ArrayList") {
                            fieldValueString = enumLists(fieldValue);
                        }
                        else if (fieldClass == "java.util.HashMap" || fieldClass == "java.util.Map") {
                            try {
                                // fieldValueString = enumMaps(fieldValue);
                                fieldValueString = fieldValue.toString();
                            }
                            catch (e) {
                                try {
                                    fieldValueString = fieldValue.toString();
                                }
                                catch (error) {
                                    fieldValueString = JSON.stringify(fieldValue);
                                }
                            }
                        }
                        else if (fieldClass == "[B") {
                            // var byteArray = Java.cast(fieldValue, Java.use("[B"));
                            // // @ts-ignore
                            // var result = Java.array('byte', byteArray);
                            fieldValueString = byteToHexString(fieldValue);
                        }
                        else {
                            try {
                                fieldValueString = fieldValue.toString();
                            }
                            catch (e) {
                                fieldValueString = JSON.stringify(fieldValue);
                            }
                        }
                    }
                    catch (e) {
                        console.log(e);
                        fieldValueString = JSON.stringify(fieldValue);
                    }
                }
                var fieldString = modifierString + " " + fieldClass + " " + fieldName + " = " + fieldValueString;
                strFields.push(fieldString);
            }
        }
    });
    return strFields;
}
function getModifierString(modifiers) {
    var modifierStr = "";
    if (Java.use("java.lang.reflect.Modifier").isPublic(modifiers)) {
        modifierStr = "public";
    }
    else if (Java.use("java.lang.reflect.Modifier").isPrivate(modifiers)) {
        modifierStr = "private";
    }
    else if (Java.use("java.lang.reflect.Modifier").isProtected(modifiers)) {
        modifierStr = "protected";
    }
    if (Java.use("java.lang.reflect.Modifier").isStatic(modifiers)) {
        modifierStr += " static";
    }
    if (Java.use("java.lang.reflect.Modifier").isFinal(modifiers)) {
        modifierStr += " final";
    }
    return modifierStr;
}
function getJnvFunctionPtr() { }
/**
 * jni 使用 jclass 获取 class name
 * @param jclz jclass
 */
function getClassNameByjclass(jnienv, jclz) {
    var name = null;
    Java.performNow(function () {
        // console.log(TAG, "[+] getClassNameByjclass step 1");
        var env = Java.vm.getEnv();
        var handlePointer = ptr(env.handle).readPointer();
        var addrCallObjectMethod = handlePointer.add(jni_struct_array["CallObjectMethod"] * Process.pointerSize).readPointer();
        var addrGetObjectClass = handlePointer.add(jni_struct_array["GetObjectClass"] * Process.pointerSize).readPointer();
        var addrAllocObject = handlePointer.add(jni_struct_array["AllocObject"] * Process.pointerSize).readPointer();
        var addrGetMethodID = handlePointer.add(jni_struct_array["GetMethodID"] * Process.pointerSize).readPointer();
        var addrGetStringUTFChars = handlePointer.add(jni_struct_array["GetStringUTFChars"] * Process.pointerSize).readPointer();
        var addrReleaseStringUTFChars = handlePointer.add(jni_struct_array["ReleaseStringUTFChars"] * Process.pointerSize).readPointer();
        var funcCallObjectMethod = null;
        var funcGetObjectClass = null;
        var funcAllocObject = null;
        var funcGetMethodID = null;
        var funcGetStringUTFChars = null;
        var funcReleaseStringUTFChars = null;
        // console.log(TAG, "[+] getClassNameByjclass step 2");
        if (addrCallObjectMethod != null && addrGetObjectClass != null) {
            funcCallObjectMethod = new NativeFunction(addrCallObjectMethod, "pointer", ["pointer", "pointer", "pointer"]);
            // void* art::JNI<true>::GetObjectClass(_JNIEnv *, _jobject *)
            funcGetObjectClass = new NativeFunction(addrGetObjectClass, "pointer", ["pointer", "pointer"]);
            // void* art::JNI<false>::AllocObject(_JNIEnv *, _jclass *)
            funcAllocObject = new NativeFunction(addrAllocObject, "pointer", ["pointer", "pointer"]);
            //
            funcGetMethodID = new NativeFunction(addrGetMethodID, "pointer", ["pointer", "pointer", "pointer", "pointer"]);
            funcGetStringUTFChars = new NativeFunction(addrGetStringUTFChars, "pointer", ["pointer", "pointer", "pointer"]);
            //
            funcReleaseStringUTFChars = new NativeFunction(addrReleaseStringUTFChars, "void", ["pointer", "pointer", "pointer"]);
            // console.log(TAG, "[+] getClassNameByjclass step 3");
        }
        if (funcCallObjectMethod != null && funcGetObjectClass != null && funcAllocObject != null && funcGetMethodID != null && funcGetStringUTFChars != null && funcReleaseStringUTFChars != null) {
            // console.log(TAG, "[+] funcCallObjectMethod: " + funcCallObjectMethod + " funcGetObjectClass: " + funcGetObjectClass);
            var clz_obj = funcAllocObject(jnienv, jclz);
            var getClassMethodID = funcGetMethodID(jnienv, jclz, Memory.allocUtf8String("getClass"), Memory.allocUtf8String("()Ljava/lang/Class;"));
            var clz_obj2 = funcCallObjectMethod(jnienv, clz_obj, getClassMethodID);
            var classClass = funcGetObjectClass(jnienv, clz_obj2);
            var getNameMethodID = funcGetMethodID(jnienv, classClass, Memory.allocUtf8String("getName"), Memory.allocUtf8String("()Ljava/lang/String;"));
            var className = funcCallObjectMethod(jnienv, clz_obj2, getNameMethodID);
            var classNamePtr = funcGetStringUTFChars(jnienv, className, ptr(0));
            name = classNamePtr.readCString();
            // console.log(TAG, "classNameCStr: " + name)
            funcReleaseStringUTFChars(jnienv, className, classNamePtr);
        }
        else {
            console.log(TAG, "[-] jni Method init fail");
        }
    });
    return name;
}
function hook_registerNativeV1() {
    Java.performNow(function () {
        var env = Java.vm.getEnv();
        var handlePointer = ptr(env.handle).readPointer();
        // console.log(TAG, '[+] handlePointer:' + handlePointer)
        var nativePointer = handlePointer.add(jni_struct_array["RegisterNatives"] * Process.pointerSize).readPointer();
        // console.log(TAG, '[+] registerNative:' + nativePointer)
        //    jint        (*RegisterNatives)(JNIEnv*, jclass, const JNINativeMethod*, jint);
        Interceptor.attach(nativePointer, {
            onEnter: function (args) {
                var nativeMethods = args[2];
                var nativeMethodCount = Number(args[3]);
                console.log(TAG, "[+] RegisterNative: class = " + getClassNameByjclass(args[0], args[1]) + "; Method Count: " + args[3]);
                for (var i = 0; i < nativeMethodCount; i++) {
                    var pMethodName = nativeMethods.add(Process.pointerSize * 3 * i).readPointer();
                    var sMethodName = pMethodName.readCString();
                    var pMethodSignature = nativeMethods.add(Process.pointerSize * 3 * i + Process.pointerSize).readPointer();
                    var sMethodSignature = pMethodSignature.readCString();
                    var pMethodAddr = nativeMethods.add(Process.pointerSize * 3 * i + Process.pointerSize * 2).readPointer();
                    var Module = Process.getModuleByAddress(pMethodAddr);
                    console.log(TAG, "[+] RegisterNative: MethodName = " + sMethodName + "; MethodSignature = " + sMethodSignature + "; MethodAddr = " + pMethodAddr.sub(Module.base).toString(16) + "; Module = " + Module.name);
                }
            },
        });
    });
}
function utf8ByteToUnicodeStr(utf8Bytes) {
    var unicodeStr = "";
    for (var pos = 0; pos < utf8Bytes.length;) {
        var flag = utf8Bytes[pos];
        var unicode = 0;
        if (flag >>> 7 === 0) {
            unicodeStr += String.fromCharCode(utf8Bytes[pos]);
            pos += 1;
        }
        else if ((flag & 0xfc) === 0xfc) {
            unicode = (utf8Bytes[pos] & 0x3) << 30;
            unicode |= (utf8Bytes[pos + 1] & 0x3f) << 24;
            unicode |= (utf8Bytes[pos + 2] & 0x3f) << 18;
            unicode |= (utf8Bytes[pos + 3] & 0x3f) << 12;
            unicode |= (utf8Bytes[pos + 4] & 0x3f) << 6;
            unicode |= utf8Bytes[pos + 5] & 0x3f;
            unicodeStr += String.fromCharCode(unicode);
            pos += 6;
        }
        else if ((flag & 0xf8) === 0xf8) {
            unicode = (utf8Bytes[pos] & 0x7) << 24;
            unicode |= (utf8Bytes[pos + 1] & 0x3f) << 18;
            unicode |= (utf8Bytes[pos + 2] & 0x3f) << 12;
            unicode |= (utf8Bytes[pos + 3] & 0x3f) << 6;
            unicode |= utf8Bytes[pos + 4] & 0x3f;
            unicodeStr += String.fromCharCode(unicode);
            pos += 5;
        }
        else if ((flag & 0xf0) === 0xf0) {
            unicode = (utf8Bytes[pos] & 0xf) << 18;
            unicode |= (utf8Bytes[pos + 1] & 0x3f) << 12;
            unicode |= (utf8Bytes[pos + 2] & 0x3f) << 6;
            unicode |= utf8Bytes[pos + 3] & 0x3f;
            unicodeStr += String.fromCharCode(unicode);
            pos += 4;
        }
        else if ((flag & 0xe0) === 0xe0) {
            unicode = (utf8Bytes[pos] & 0x1f) << 12;
            unicode |= (utf8Bytes[pos + 1] & 0x3f) << 6;
            unicode |= utf8Bytes[pos + 2] & 0x3f;
            unicodeStr += String.fromCharCode(unicode);
            pos += 3;
        }
        else if ((flag & 0xc0) === 0xc0) {
            //110
            unicode = (utf8Bytes[pos] & 0x3f) << 6;
            unicode |= utf8Bytes[pos + 1] & 0x3f;
            unicodeStr += String.fromCharCode(unicode);
            pos += 2;
        }
        else {
            unicodeStr += String.fromCharCode(utf8Bytes[pos]);
            pos += 1;
        }
    }
    return unicodeStr;
}
function uuid() {
    var s = [];
    var hexDigits = "0123456789abcdef";
    for (var i = 0; i < 36; i++) {
        s[i] = hexDigits.substr(Math.floor(Math.random() * 0x10), 1);
    }
    s[14] = "4"; // bits 12-15 of the time_hi_and_version field to 0010
    var clockSeqHi = parseInt(s[19], 16);
    clockSeqHi = (clockSeqHi & 0x3) | 0x8;
    s[19] = clockSeqHi.toString(16);
    s[8] = s[13] = s[18] = s[23] = "-";
    var uuid = s.join("");
    // 去掉 “-”
    // uuid = uuid.split("-").join("");
    return uuid;
}
function stringToByte(str) {
    var javaString = Java.use("java.lang.String");
    var bytes = [];
    bytes = javaString.$new(str).getBytes();
    return bytes;
}
function byteToString(byte) {
    var javaString = Java.use("java.lang.String");
    return javaString.$new(byte);
}
/**
 *
 * @param bytes byte[] 数组
 * @description 将 byte[] 转换为十六进制字符串
 * @example
 * var bytes = [0x01, 0x02, 0x03, 0x04];
 * var hexString = byteToHexString(bytes);
 * console.log(hexString); // 输出: "01 02 03 04"
 * @returns
 */
function byteToHexString(bytes) {
    let bytesCls = Java.use("[B");
    var byteValues = [];
    // 必须转一下，不然 java.lang.Object 传过来的 [B 类型读不出来
    if (bytesCls.class.isInstance(bytes)) {
        bytes = Java.cast(bytes, Java.use("[B"));
        var ArrayClass = Java.use("java.lang.reflect.Array");
        var length = ArrayClass.getLength(bytes);
        for (var i = 0; i < length; i++) {
            var byteValue = ArrayClass.get(bytes, i);
            byteValues.push(byteValue);
        }
    }
    else {
        return "";
    }
    if (byteValues == null || byteValues.length == 0) {
        return "";
    }
    var hexArray = "0123456789ABCDEF";
    var hexChars = [];
    for (var j = 0; j < byteValues.length; j++) {
        var v = byteValues[j] & 0xff;
        hexChars[j * 3] = hexArray[v >>> 4];
        hexChars[j * 3 + 1] = hexArray[v & 0x0f];
        hexChars[j * 3 + 2] = " ";
    }
    // 移除最后一个空格
    if (hexChars.length > 0) {
        hexChars.pop();
    }
    return Java.use("java.lang.String").$new(hexChars);
}
function enableWebviewDebug() {
    Java.perform(function () {
        var Webview = Java.use("android.webkit.WebView");
        Webview.loadUrl.overload("java.lang.String").implementation = function (url) {
            // // console.log("\n[+]Loading URL from", url);
            // console.log(Java.use("android.util.log").getStackTraceString(Java.use("java.lang.Throwable").$new()));
            // console.log(Java.use("android.util.Log").getStackTraceString(Java.use("java.lang.Exception").$new()));
            console.log("[+]Setting the value of setWebContentsDebuggingEnabled() to TRUE");
            this.setWebContentsDebuggingEnabled(true);
            this.loadUrl.overload("java.lang.String").call(this, url);
        };
    });
}
/**
 * 解压 zip 压缩过的 byte[]
 * @param compressedBytes 压缩的 byte[]
 * @returns
 */
function inflaterByteArray(compressedBytes) {
    var ByteArrayInputStream = Java.use("java.io.ByteArrayInputStream");
    var bais = ByteArrayInputStream.$new(compressedBytes);
    var InflaterInputStream = Java.use("java.util.zip.InflaterInputStream");
    var inflaterInputStream = InflaterInputStream.$new(bais);
    var ByteArrayOutputStream = Java.use("java.io.ByteArrayOutputStream");
    var baos = ByteArrayOutputStream.$new();
    var buffer = Java.array("byte", Array(4096).fill(0));
    var readMethod = inflaterInputStream.read.overload("[B");
    while (true) {
        var len = readMethod.call(inflaterInputStream, buffer);
        if (len == -1)
            break;
        baos.write(buffer, 0, len);
    }
    var result = baos.toByteArray();
    // 转成 JS 数组
    var resultArray = [];
    for (var i = 0; i < result.length; i++) {
        resultArray.push(result[i]);
    }
    // 关闭流
    baos.close();
    inflaterInputStream.close();
    bais.close();
    return resultArray;
    // 输出解压后的数据
    console.log("解压后数据长度: " + resultArray.length);
    // 如需输出内容，可用 base64 或 hex
    console.log("解压后内容: " + resultArray);
}
/**
 * 封装：读取 InputStream 内容但不消耗，并返回新流和内容
 * @param inputStream
 * @returns
 */
function readInputStreamWithoutConsuming(inputStream) {
    var ByteArrayOutputStream = Java.use("java.io.ByteArrayOutputStream");
    var ByteArrayInputStream = Java.use("java.io.ByteArrayInputStream");
    var buffer = Java.array("byte", Array(4096).fill(0));
    var baos = ByteArrayOutputStream.$new();
    var readMethod = inputStream.read.overload("[B");
    while (true) {
        var len = readMethod.call(inputStream, buffer);
        if (len == -1)
            break;
        baos.write(buffer, 0, len);
    }
    var allBytes = baos.toByteArray();
    var newInputStream = ByteArrayInputStream.$new(allBytes);
    return { newInputStream: newInputStream, content: allBytes };
}
function getJavaStackTraceString() {
    var stackRead;
    Java.perform(() => {
        stackRead = Java.use("android.util.Log").getStackTraceString(Java.use("java.lang.Throwable").$new()).toString();
    });
    return stackRead;
}
// export function enableWebviewDebug()
// {
//     Java.perform(function ()
//     {
//         var WebView = Java.use("android.webkit.WebView");
//         WebView.setWebContentsDebuggingEnabled.overload("boolean").implementation = function (s:any) {
//         // send(s.toString());
//         console.log("webview hook")
//         // this.loadUrl.overload("java.lang.String").call(this, s);
//         // console.log(this.)
//         this.setWebContentsDebuggingEnabled(true)
//         };
//     });
// }
// _dl__ZN6soinfo17call_constructorsEv
// Module.getExportByName("linker64", "__dl__ZN6soinfo17call_constructorsEv")
/**
 *
 * @param offset_of_call_constructors call_constructors 的 address
 * @param hook_infos
 * @returns
 */
function hook_callConstructors(offset_of_call_constructors, hook_infos) {
    var pcall_constructors = ptr(0);
    var module_base_linker = null;
    if (Process.arch == "arm") {
        module_base_linker = Module.findBaseAddress("linker");
        pcall_constructors = pcall_constructors.add(1);
    }
    else if (Process.arch == "arm64") {
        module_base_linker = Module.findBaseAddress("linker64");
    }
    if (module_base_linker == null) {
        console.log(TAG, "[-] didn't find linker!!!\n");
        return;
    }
    pcall_constructors = module_base_linker.add(offset_of_call_constructors).add(pcall_constructors);
    console.log(TAG, Process.arch + "\tLinker addr: " + module_base_linker + "\tFind CallConstructors: " + pcall_constructors + "\n");
    Interceptor.attach(pcall_constructors, {
        onEnter: function (args) {
            // console.log(hexdump(args[0]));
            var md = null;
            if (Process.arch == "arm") {
                var md_name = args[0].readCString();
                if (md_name != null) {
                    md = Process.findModuleByName(md_name);
                }
                else {
                    const SOINFO_NAME_LEN = 128;
                    var offset_of_module_addr = args[0]
                        .add(SOINFO_NAME_LEN)
                        .add(Process.pointerSize * 4)
                        .readPointer();
                    md = Process.findModuleByAddress(offset_of_module_addr);
                }
            }
            else if (Process.arch == "arm64") {
                var base = args[0].add(Process.pointerSize * 2).readPointer();
                md = Process.findModuleByAddress(base);
            }
            if (md != null) {
                console.log(TAG, "soinfo.name: " + md.name + ", soinfo.base: " + md.base + ", soinfo.size: " + md.size + "\n");
                if (hook_infos != null) {
                    // @ts-ignore
                    const hook_info = hook_infos.find((info) => info.so_name === md.name);
                    if (hook_info) {
                        this.hook_info = hook_info;
                        this.md = md;
                        console.log(TAG, "soinfo.name: " + md.name + ", soinfo.base: " + md.base + ", soinfo.size: " + md.size + "\n");
                        if (hook_info.onEnter_hook_func) {
                            hook_info.onEnter_hook_func(md);
                        }
                    }
                }
            }
        },
        onLeave: function (ret) {
            if (this.hook_info && this.md) {
                if (this.hook_info.onLeave_hook_func) {
                    this.hook_info.onLeave_hook_func(this.md);
                }
            }
        },
    });
}
function initializeNativeFunction(fname, retType, argTypes) {
    const p = Module.findExportByName(null, fname);
    if (p !== null) {
        return new NativeFunction(p, retType, argTypes);
    }
    return null;
}
function fopen(filePath, perm) {
    const fopenImpl = initializeNativeFunction("fopen", "pointer", ["pointer", "pointer"]);
    const filePathPtr = Memory.allocUtf8String(filePath);
    const p = Memory.allocUtf8String(perm);
    if (fopenImpl) {
        //@ts-ignore
        return fopenImpl(filePathPtr, p);
    }
    console.log(TAG, "didn't find fopen!");
    return NULL;
}
function getStackTraceString(context, tab) {
    var tabstr = "";
    for (var i = 0; i < tab; i++) {
        tabstr += " ";
    }
    var traces = Thread.backtrace(context, Backtracer.FUZZY);
    return traces
        .map(function (curval, idx, arr) {
        var mod = Process.findModuleByAddress(curval);
        let ret = {};
        if (!mod) {
            ret.name = "null";
            ret.base = "0x00000000";
            ret.offs = "0x" + curval.toString(16).toUpperCase();
        }
        else {
            ret.name = mod.name;
            ret.base = "0x" + mod.base.toString(16).toUpperCase();
            ret.offs = "0x" + curval.sub(mod.base).toString(16).toUpperCase();
        }
        return JSON.stringify(ret);
    })
        .join("\n" + tabstr);
}
function patch_exit(md) {
    const pattern = "28 10 80 D2 01 00 00 D4";
    // console.log(hexdump(md.base.add(0x323f0)))
    // console.log('md:\n' + JSON.stringify(md));
    const results = Memory.scanSync(md.base, md.size, pattern); // todo: 可能扫到不可访问的地址
    results.forEach((item, index) => {
        const maxPatchSize = 8; // Do not write out of bounds, may be a temporary buffer!
        Memory.patchCode(item.address, maxPatchSize, (code) => {
            console.log(TAG, "[+]\t patch exit svc: " + item.address);
            const cw = new Arm64Writer(code, { pc: item.address });
            cw.putNop();
            cw.putNop();
            cw.flush();
        });
    });
}
// todo: TypeError: cannot read property 'getApplicationContext' of null
function getApkFilePath() {
    var dir = "";
    if (Java.available) {
        Java.performNow(function () {
            var currentApplication = Java.use("android.app.ActivityThread").currentApplication();
            dir = currentApplication.getApplicationContext().getFilesDir().getPath();
        });
    }
    return dir;
}
/**
 * hook 某地址，打印当前地址的 CpuContext
 * @param addr
 * @returns -1 | 0
 */
function printSpecificAddrCpuContext(tag, addr) {
    if (addr == null) {
        console.log(`[${tag}]`, "addr is null");
        return -1;
    }
    Interceptor.attach(addr, {
        onEnter: function (args) {
            console.log(`[${tag}]`, JSON.stringify(this.context));
        },
    });
    return 0;
}
// 用于 dump rw- 内存段
function dumpMemoryRanges(prefix, outputDir = "/data/local/tmp") {
    const ranges = Process.enumerateRanges({
        protection: "rw-",
        coalesce: false,
    });
    ranges.forEach((r) => {
        try {
            const fname = `${outputDir}/${prefix}_${r.base.toString()}_${r.base.add(r.size)}_mem.bin`;
            dumpMemory(r.base, r.size, fname);
        }
        catch (error) {
            console.error(`[-]\tFailed dump ${r.base}:`, error);
        }
    });
}
/**
 * 简单附加到该地址，输出该地址的信息
 * @param offset so 的偏移
 * @param md_name 模块名字
 */
function confirmAddress(offset, md_name) {
    var md = Process.findModuleByName(md_name);
    if (md == null) {
        console.log(TAG, "[-]", `couldn't find ${offset} in ${md_name}`);
        return;
    }
    var fun_ptr = md.base.add(offset);
    Interceptor.attach(fun_ptr, {
        onEnter: function (args) {
            console.log(`${TAG}\t${md_name}: ${offset} -> ${JSON.stringify(this.context)} \n${getStackTraceString(this.context, 4)}}`);
            // getStackTraceString(this.context, 4);
        },
    });
}
function dumpThreadRegs(prefix, cpu_context, outputDir = "/data/local/tmp") {
    const json = JSON.stringify(cpu_context, null, 2);
    var filePath = "";
    filePath = `${outputDir}/${prefix}_regs.json`;
    console.log(`${filePath}`);
    const f = new File(filePath, "w");
    f.write(json);
    f.flush();
    f.close();
    console.log(`📝 Saved ${cpu_context.pc} register context: ${filePath}`);
}
function createApplicationContextInner() {
    return new Promise((resolve) => {
        var Handler = Java.use("android.os.Handler");
        const HCallback = Java.use("android.os.Handler$Callback");
        var Message = Java.use("android.os.Message");
        var ActivityThread = Java.use("android.app.ActivityThread");
        var Looper = Java.use("android.os.Looper");
        var MyCallback = null;
        // 定义一个回调来拦截 handleMessage
        Java.perform(() => {
            MyCallback = Java.registerClass({
                name: "com.example.MyCallback",
                implements: [HCallback],
                methods: {
                    handleMessage(msg) {
                        // console.log('Message what: ' + msg.what.value);
                        if (msg.what.value === 1) {
                            var currentApplication = ActivityThread.currentApplication();
                            var context = currentApplication.getApplicationContext();
                            // console.log(TAG, "context: " + JSON.stringify(context))
                            resolve(Java.retain(context));
                            return true;
                        }
                        return false;
                    },
                },
            });
            if (MyCallback != null) {
                Java.scheduleOnMainThread(() => {
                    var myCallback = MyCallback.$new();
                    var mainLooper = Looper.getMainLooper();
                    var handler = Handler.$new(mainLooper, myCallback);
                    var message = handler.obtainMessage(1);
                    handler.sendMessage(message);
                });
            }
            else {
                console.log("register callback failed");
            }
        });
    });
}
var gContext = null;
async function createApplicationContext() {
    if (gContext == null) {
        gContext = await createApplicationContextInner();
        // console.log("createApplicationContextImpl: ", JSON.stringify(gContext))
    }
    // console.log("createApplicationContextImpl: ", JSON.stringify(gContext))
    return gContext;
}
// 在同一个 vm 里（在 Java.perform() 中调用这个函数）
function getApplicationContext() {
    var ActivityThread = Java.use("android.app.ActivityThread");
    var app = ActivityThread.currentApplication();
    var context = app.getApplicationContext();
    return context;
}
function getPackageName() {
    Java.perform(function () {
        var ActivityThread = Java.use("android.app.ActivityThread");
        var app = ActivityThread.currentApplication();
        var context = app.getApplicationContext();
        var pm = context.getPackageManager();
        var pi = pm.getPackageInfo(context.getPackageName(), 0);
        console.log(TAG, "Package Name: " + pi.packageName);
        return pi.packageName;
    });
    return null;
}
function getDataPath(pkgName) {
    // var pkgName = getPackageName();
    if (pkgName) {
        Java.performNow(() => {
            var pattern = /\d+/;
            var name = Java.use("android.os.Environment").getExternalStorageDirectory().getName();
            if (name.match(pattern) != null) {
                var path = "/data/user/" + name + "/" + pkgName;
                return path;
            }
        });
        var path = "/data/data/" + pkgName;
        return path;
    }
    return null;
}
function isPointerReadableWritable(ptr) {
    const range = Process.findRangeByAddress(ptr);
    return range !== null && range.protection.indexOf("r") !== -1 && range.protection.indexOf("w") !== -1;
}
function find_symbols_by_addr(addr) {
    var method_info = null;
    var func_mod = Process.findModuleByAddress(addr);
    if (func_mod) {
        var exports = func_mod.enumerateExports();
        var inports = func_mod.enumerateImports();
        for (var k = 0, length = exports.length; k < length; k++) {
            // console.log("[=]\tenum symbol: " + exports[k].name + ": " + exports[k].address + " = " + addr)
            if (Number(exports[k].address) == Number(addr)) {
                // console.log("[+]\tfind symbol: " + exports[k].name + ": " + exports[k].address + " = " + addr)
                method_info = exports[k];
                break;
            }
        }
        if (!method_info) {
            for (var k = 0, length = inports.length; k < length; k++) {
                // console.log("[=]\tenum symbol: " + inports[k].name + ": " + inports[k].address + " = " + addr)
                if (Number(inports[k].address) == Number(addr)) {
                    // console.log("[+]\tfind symbol: " + inports[k].name + ": " + inports[k].address + " = " + addr)
                    method_info = inports[k];
                    break;
                }
            }
        }
    }
    return method_info;
}
function createFolder(path, mode) {
    try {
        var mkdir = Module.findExportByName(null, "mkdir");
        if (!mkdir) {
            console.error(TAG, `[-]\tCannot find mkdir function`);
            return;
        }
        var mkdirFunc = new NativeFunction(mkdir, "int", ["pointer", "int"]);
        var pathPtr = Memory.allocUtf8String(path);
        var result = mkdirFunc(pathPtr, mode);
        if (result === 0) {
            console.log(TAG, `[+]\tFolder created at ${path}`);
        }
        else {
            console.error(TAG, `[-]\tFailed to create folder: ${path}, errno: ${result}`);
        }
    }
    catch (e) {
        console.error(TAG, `[-]\tException: ${e}`);
    }
}
function generateRandomString(length) {
    const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let result = "";
    for (let i = 0; i < length; i++) {
        result += characters.charAt(Math.floor(Math.random() * characters.length));
    }
    return result;
}
function dumpSo(so_name, path) {
    if (path == null) {
        path = getApkFilePath();
    }
    var libso = Process.getModuleByName(so_name);
    console.log(TAG, "[name]:", libso.name);
    console.log(TAG, "[base]:", libso.base);
    console.log(TAG, "[size]:", ptr(libso.size));
    console.log(TAG, "[path]:", libso.path);
    console.log(TAG, "[save path]:", path);
    // if (path == "")
    // {
    //     path = libso.path.substring(0, libso.path.lastIndexOf("/"));
    // }
    var file_path = path + "/" + libso.name + "_" + libso.base + "_" + ptr(libso.size) + ".so";
    console.log(TAG, "[+] save to: ", file_path);
    dumpMemory(libso.base, libso.size, file_path);
}
/**
 * dump java byte[]
 *     fname: dump 文件名，/data/data/<你的应用包名>/<fname>
 *     data: byte[]
 */
function dumpJavaU8(fname, data) {
    try {
        const context = Java.use("android.app.ActivityThread").currentApplication().getApplicationContext();
        const filePath = context.getFilesDir().getPath() + "/" + fname;
        console.log(TAG, `Start write ${data.length} data to ${filePath}`);
        const fos = Java.use("java.io.FileOutputStream").$new(filePath);
        fos.write(data);
        fos.close();
        console.log(TAG, `write ${data.length} data success!`);
    }
    catch (e) {
        console.error(TAG, e);
    }
}
function jsArrayBufferToJavaByteArray(buffer) {
    var jsBuffer = new Uint8Array(buffer);
    const javaBytes = Java.array("byte", Array.from(jsBuffer));
    return javaBytes; // 2 = NO_WRAP
}
/**
 * todo!
 * @param javaByteArray
 * @returns
 */
function javaByteToJsArrayBufferV2(javaByteArray) {
    if (!javaByteArray)
        return null;
    const len = javaByteArray.length;
    if (len === 0)
        return new ArrayBuffer(0);
    try {
        const ByteBuffer = Java.use("java.nio.ByteBuffer");
        // 第1步：分配 DirectByteBuffer
        const directBuf = ByteBuffer.allocateDirect(len);
        // 第2步：把 byte[] 内容复制进去（这步是唯一一次 memcpy，速度极快）
        directBuf.put(javaByteArray);
        // 第3步：关键！用 Frida 的新 API 直接取 DirectBuffer 的地址
        // 从 Frida 15.1.17+ 开始官方加了这个超级好用的 wrapper
        const address = directBuf.$address;
        if (!address || address.isNull()) {
            throw new Error("Failed to get direct buffer address");
        }
        // 第4步：直接读内存 → 得到真正的 ArrayBuffer（零拷贝！）
        const arrayBuffer = address.readByteArray(len);
        if (!arrayBuffer) {
            throw new Error("readByteArray returned null");
        }
        console.log(`[√] java.byte[] → ArrayBuffer 成功，${len} bytes (zero-copy)`);
        return arrayBuffer;
    }
    catch (e) {
        console.error("[×] DirectBuffer 方案失败，回退到 MemoryAccess 方案", e);
        return javaByteToJsArrayBuffer(javaByteArray);
    }
}
function javaByteToJsArrayBuffer(javaByteArray) {
    // 强制转换为 byte 数组类型，确保可以使用索引访问
    var Byte = Java.use("java.lang.Byte");
    // 假设 javaByteArray 是你的对象
    // 如果它是 Object 类型，需要先 cast: var arr = Java.cast(obj, Java.use("[B"));
    var jsArray = [];
    // 注意：Frida 的 Java 数组对象可以直接通过 .length 和下标访问
    for (var i = 0; i < javaByteArray.length; i++) {
        // Java byte 是有符号的 (-128 到 127)，为了转为无符号 byte，通常需要 & 0xFF
        // 但如果只是为了放入 ArrayBuffer，TypedArray 会自动处理
        jsArray.push(javaByteArray[i]);
    }
    // 生成 Uint8Array (ArrayBuffer 的视图)
    var uint8Arr = new Uint8Array(jsArray);
    var arrayBuffer = uint8Arr.buffer;
    return arrayBuffer;
}
function dumpMemory(start, size, path) {
    var base;
    if (typeof start == "number") {
        base = ptr(start);
    }
    else {
        base = start;
    }
    var file_handle = new File(path, "wb");
    if (file_handle && file_handle != null) {
        Memory.protect(base, size, "rwx");
        var libso_buffer = base.readByteArray(size);
        if (libso_buffer != null) {
            file_handle.write(libso_buffer);
            file_handle.flush();
            file_handle.close();
            console.log(TAG, "[dump][+] base:", base, "\tsize:", size, "\tpath:", path);
        }
        else {
            console.log(TAG, "[dump][-] base:", base, "\tsize:", size, "\tpath:", path, " failed");
        }
    }
}
function find_module_by_function_name(name) {
    var target_module = null;
    var addr = Module.findExportByName(null, name);
    if (addr == null) {
        return null;
    }
    else {
        target_module = Process.findModuleByAddress(addr);
    }
    return target_module;
}
function readMaps() {
    var maps = [];
    var path = "/proc/" + Process.id + "/maps";
    console.log(TAG, "reading " + path);
    var file_handle = new File(path, "r");
    var line;
    while (((line = file_handle.readLine()), line.length != 0)) {
        maps.push(line);
        // console.log(TAG, line);
    }
    file_handle.close();
    return maps;
}
function readStdString(str) {
    const isTiny = (str.readU8() & 1) == 0;
    if (isTiny) {
        return str.add(1).readUtf8String();
    }
    return str
        .add(2 * Process.pointerSize)
        .readPointer()
        .readUtf8String();
}
function scanMemorySync(pattern, startAddr, endAddr, maxMatches, context) {
    try {
        var ranges = Process.enumerateRanges("r--");
        var matches = [];
        var stopped = false;
        if (ranges.length === 0) {
            console.log(TAG, "[scan] [-]\tNo readable memory ranges found");
            return;
        }
        for (var i = 0; i < ranges.length && !stopped && matches.length < maxMatches; i++) {
            var range = ranges[i];
            if (startAddr && range.base.compare(startAddr) < 0)
                continue;
            if (endAddr && range.base.add(range.size).compare(endAddr) > 0)
                continue;
            try {
                var results = Memory.scanSync(range.base, range.size, pattern);
                for (var j = 0; j < results.length && matches.length < maxMatches; j++) {
                    var match = results[j];
                    console.log(TAG, `[scan] match: ${match.address.toString()}, size: ${match.size}`);
                    matches.push(match.address);
                }
            }
            catch (e) {
                console.error(TAG, "[scan] [-]\tScan error in range " + range.base + ": " + e);
            }
        }
        if (matches.length === 0) {
            console.log(TAG, "[scan] [-]\tNo matches found");
        }
        else {
            console.log(TAG, "[scan] [+]\t" + matches.length + " matches found\n" + getStackTraceString(context, 4));
            matches.forEach(function (match) {
                console.log(TAG, "[scan] [+]\t" + hexdump(match, { length: 0x20 }));
            });
        }
    }
    catch (e) {
        console.error(TAG, "[scan] [-]\t" + "Exception: " + e);
    }
}
function scanMemory(pattern, startAddr, endAddr, maxMatches) {
    try {
        var ranges = Process.enumerateRanges("rw-");
        var matches = [];
        var stopped = false;
        var pendingScans = 0;
        if (ranges.length === 0) {
            console.log(TAG, "[scan] [-]\tNo readable memory ranges found");
            return;
        }
        ranges.forEach(function (range) {
            if (stopped || matches.length >= maxMatches) {
                return;
            }
            if (startAddr && range.base.compare(startAddr) < 0) {
                return;
            }
            if (endAddr && range.base.add(range.size).compare(endAddr) > 0) {
                return;
            }
            pendingScans++;
            Memory.scan(range.base, range.size, pattern, {
                onMatch: function (address, size) {
                    if (stopped || matches.length >= maxMatches) {
                        stopped = true;
                        return "stop";
                    }
                    console.log(TAG, `[scan] match: ${address.toString()}, size: ${size}`);
                    matches.push(address);
                    if (matches.length >= maxMatches) {
                        stopped = true;
                        return "stop";
                    }
                },
                onError: function (reason) {
                    console.error(TAG, "[scan] [-]\tScan error in range " + range.base + ": " + reason);
                    if (--pendingScans === 0 && !stopped) {
                        if (matches.length === 0) {
                            console.log(TAG, "[scan] [-]\tNo matches found");
                        }
                        else {
                            console.log(TAG, "[scan] [+]\t" + matches.length + " matches found");
                            matches.forEach(function (match) {
                                console.log(TAG, "[scan] [+]\t" + hexdump(match, { length: 20 }));
                            });
                        }
                    }
                },
                onComplete: function () {
                    if (--pendingScans === 0 && !stopped) {
                        if (matches.length === 0) {
                            console.log(TAG, "[scan] [-]\tNo matches found");
                        }
                        else {
                            console.log(TAG, "[scan] [+]\t" + matches.length + " matches found");
                            matches.forEach(function (match) {
                                console.log(TAG, "[scan] [+]\t" + hexdump(match, { length: 20 }));
                            });
                        }
                    }
                },
            });
        });
        if (pendingScans === 0) {
            console.log(TAG, "[scan] [+]\tNo valid memory ranges to scan");
            return;
        }
    }
    catch (e) {
        console.error("Exception: " + e);
    }
}
/**
 * default 是 anti 爱加密的检测
 * @param target_so map 的 第一个参数为 so 的名字，第二个参数为这个 so 中想清除的线程地址数组
 */
function antiAnti(target_so) {
    var default_so = new Map([
        ["libexecmain.so", null],
        ["libexec.so", null],
        ["libijmdetect-drisk.so", null],
    ]);
    target_so == null ? (target_so = default_so) : (target_so = Object.assign(target_so, default_so));
    var pthread_create_addr = Module.findExportByName("libc.so", "pthread_create");
    if (pthread_create_addr) {
        // console.log("pthread_create_addr: ", pthread_create_addr);
        Interceptor.attach(pthread_create_addr, {
            onEnter: function (args) {
                // @ts-ignore
                for (var so of target_so) {
                    var mod = Process.findModuleByAddress(args[2]);
                    // console.log(TAG, "so name: " + so + " mod name: " + (mod == null ? null : mod.name));
                    if (mod && mod.name == so[0]) {
                        if (so[1] == null) {
                            console.log(TAG, "so name: " + so + " mod name: " + (mod == null ? null : mod.name));
                            Interceptor.replace(args[2], new NativeCallback(function () {
                                console.log(TAG, "close thread: ");
                            }, "void", ["void"]));
                        }
                        else {
                            for (var addr of so[1]) {
                                if (args[2].equals(addr.add(mod.base))) {
                                    console.log(TAG, mod?.name, addr.add(mod.base), args[2], args[2].equals(addr.add(mod.base)));
                                    // Interceptor.attach(args[2], () => {
                                    //     // @ts-ignore
                                    //     console.log(TAG, args[2], this.context.lr);
                                    // })
                                    Interceptor.replace(args[2], new NativeCallback(function () {
                                        console.log(TAG, "close thread: ");
                                        // console.log("test");
                                        // console.log(TAG, mod?.name + ": close thread: " + args[2])
                                        // console.log(TAG, mod?.name + ": close thread: " + args[2]);
                                    }, "void", ["void"]));
                                }
                            }
                        }
                    }
                }
            },
            onLeave: function (retval) { },
        });
    }
}
/**
 * bool JavaVMExt::LoadNativeLibrary(JNIEnv* env,
                                     const std::string& path,
                                     jobject class_loader,
                                     jclass caller_class,
                                     std::string* error_msg)
 *
 */
function hook_LoadNativeLibrary(fnOnEnter, fnOnLeave) {
    // art::JavaVMExt::LoadNativeLibrary(_JNIEnv *, std::string const&, _jobject *, _jclass *, std::string*)
    // _ZN3art9JavaVMExt17LoadNativeLibraryEP7_JNIEnvRKNSt3__112basic_stringIcNS3_11char_traitsIcEENS3_9allocatorIcEEEEP8_jobjectP7_jclassPS9_
    var LoadNativeLibrary_addr = Module.findExportByName("libart.so", "_ZN3art9JavaVMExt17LoadNativeLibraryEP7_JNIEnvRKNSt3__112basic_stringIcNS3_11char_traitsIcEENS3_9allocatorIcEEEEP8_jobjectP7_jclassPS9_");
    if (LoadNativeLibrary_addr != null) {
        Interceptor.attach(LoadNativeLibrary_addr, {
            onEnter(args) {
                this.path = readStdString(args[2]);
                console.log(TAG, this.path);
                if (fnOnEnter)
                    fnOnEnter(this.path);
                //console.log(TAG);
                // console.log(TAG, args[2], "\n");
                // console.log(hexdump(args[2]), "\n")
                //console.log(hexdump(args[2].add(2*Process.pointerSize).readPointer()));
                // console.log(TAG, args[2], "\n", hexdump(args[2]), "\n",
                //             hexdump(args[2].readPointer(), "\n", ), "\n",
                //             hexdump(args[2].add(1 * Process.pointerSize).readPointer()), "\n",
                //             hexdump(args[2].add(2 * Process.pointerSize).readPointer()), "\n",);
                // console.log(TAG, hexdump(args[1]));
            },
            onLeave(retval) {
                if (fnOnLeave)
                    fnOnLeave(this.path);
            },
        });
    }
    // art::JavaVMExt::LoadNativeLibrary(_JNIEnv *, std::string const&, _jobject *, _jclass *, std::string*)::$_16::operator()(_jobject *)const
    // _ZZN3art9JavaVMExt17LoadNativeLibraryEP7_JNIEnvRKNSt3__112basic_stringIcNS3_11char_traitsIcEENS3_9allocatorIcEEEEP8_jobjectP7_jclassPS9_ENK4$_16clESD_
    LoadNativeLibrary_addr = Module.findExportByName("libart.so", "_ZZN3art9JavaVMExt17LoadNativeLibraryEP7_JNIEnvRKNSt3__112basic_stringIcNS3_11char_traitsIcEENS3_9allocatorIcEEEEP8_jobjectP7_jclassPS9_ENK4$_16clESD_");
    if (LoadNativeLibrary_addr != null) {
        Interceptor.attach(LoadNativeLibrary_addr, {
            onEnter(args) {
                console.log(TAG, readStdString(args[2]));
                if (fnOnEnter)
                    fnOnEnter();
            },
            onLeave(retval) {
                if (fnOnLeave)
                    fnOnLeave();
            },
        });
    }
}
function enumMaps(hashMapInstance) {
    var result = "\n\{\n";
    Java.perform(function () {
        hashMapInstance = Java.cast(hashMapInstance, Java.use("java.util.HashMap"));
        var HashMap = Java.use("java.util.HashMap"); // 替换为你的实际类路径
        var entrySet = hashMapInstance.entrySet();
        var entryIterator = entrySet.iterator();
        while (entryIterator.hasNext()) {
            var entry = Java.cast(entryIterator.next(), Java.use("java.util.Map$Entry"));
            var key = entry.getKey();
            var value = entry.getValue();
            // [B 类型输出
            let bytesCls = Java.use("[B");
            if (bytesCls.class.isInstance(value)) {
                value = byteToHexString(value);
            }
            result += "\t" + key + "\t=>\t" + value + "\n";
            // console.log('Key: ' + key + ', Value: ' + firstByte);
        }
        result += "\}";
    });
    return result;
}
function enumLists(listInstance) {
    var result = "[";
    Java.perform(function () {
        var ArrayList = Java.use("java.util.ArrayList"); // 替换为你的实际类路径
        var iterator = listInstance.iterator();
        while (iterator.hasNext()) {
            var element = iterator.next();
            result += element.toString() + ", ";
        }
        result += "]";
    });
    return result;
}
// module.exports = {
//     hook_registerNative: hook_registerNative,
//     GetStackTraceString: getStackTraceString,
//     dump_so: dumpSo,
//     hook_call_constructors:hook_callConstructors
// }

},{}],2:[function(require,module,exports){
(function (Buffer){(function (){
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.HookSSL = HookSSL;
/**
 * Initializes 'addresses' dictionary and NativeFunctions.
 */
// "use strict";
// rpc.exports = {
//   setssllib: function (name) {
//     console.log("setSSLLib => " + name);
//     libname = name;
//     initializeGlobals();
//     return;
//   },
// };
const traffic_analyzer_1 = require("./traffic_analyzer");
const analyzer = new traffic_analyzer_1.TrafficAnalyzer();
var addresses = {};
var SSL_get_fd = null;
var SSL_get_session = null;
var SSL_SESSION_get_id = null;
var getpeername = null;
var getsockname = null;
var ntohs = null;
var ntohl = null;
var SSLstackwrite = null;
var SSLstackread = null;
var libname = "*libssl*";
function return_zero(args) {
    return 0;
}
function initializeGlobals() {
    var resolver = new ApiResolver("module");
    var exps = [
        [Process.platform == "darwin" ? "*libboringssl*" : "*libssl*", ["SSL_read", "SSL_write", "SSL_get_fd", "SSL_get_session", "SSL_SESSION_get_id"]], // for ios and Android
        [Process.platform == "darwin" ? "*libsystem*" : "*libc*", ["getpeername", "getsockname", "ntohs", "ntohl"]],
    ];
    // console.log(exps)
    for (var i = 0; i < exps.length; i++) {
        var lib = exps[i][0];
        var names = exps[i][1];
        for (var j = 0; j < names.length; j++) {
            var name = names[j];
            // console.log("exports:" + lib + "!" + name)
            var matches = resolver.enumerateMatches("exports:" + lib + "!" + name);
            if (matches.length == 0) {
                if (name == "SSL_get_fd") {
                    addresses["SSL_get_fd"] = ptr(0);
                    continue;
                }
                throw "Could not find " + lib + "!" + name;
            }
            else if (matches.length != 1) {
                // Sometimes Frida returns duplicates.
                var address = ptr(0);
                var s = "";
                var duplicates_only = true;
                for (var k = 0; k < matches.length; k++) {
                    if (s.length != 0) {
                        s += ", ";
                    }
                    s += matches[k].name + "@" + matches[k].address;
                    if (address == ptr(0)) {
                        address = matches[k].address;
                    }
                    else if (!address.equals(matches[k].address)) {
                        duplicates_only = false;
                    }
                }
                if (!duplicates_only) {
                    throw "More than one match found for " + lib + "!" + name + ": " + s;
                }
            }
            addresses[name] = matches[0].address;
        }
    }
    if (addresses["SSL_get_fd"] == ptr(0)) {
        SSL_get_fd = return_zero;
    }
    else {
        SSL_get_fd = new NativeFunction(addresses["SSL_get_fd"], "int", ["pointer"]);
    }
    SSL_get_session = new NativeFunction(addresses["SSL_get_session"], "pointer", ["pointer"]);
    SSL_SESSION_get_id = new NativeFunction(addresses["SSL_SESSION_get_id"], "pointer", ["pointer", "pointer"]);
    getpeername = new NativeFunction(addresses["getpeername"], "int", ["int", "pointer", "pointer"]);
    getsockname = new NativeFunction(addresses["getsockname"], "int", ["int", "pointer", "pointer"]);
    ntohs = new NativeFunction(addresses["ntohs"], "uint16", ["uint16"]);
    ntohl = new NativeFunction(addresses["ntohl"], "uint32", ["uint32"]);
    if (SSL_get_fd == null || SSL_SESSION_get_id == null || SSL_get_session == null || getsockname == null || getpeername == null || ntohs == null || ntohl == null) {
        console.log("Failed to load SSL functions");
        return false;
    }
}
traffic_analyzer_1.TrafficAnalyzer;
function ipToNumber(ip) {
    var num = 0;
    if (ip == "") {
        return num;
    }
    var aNum = ip.split(".");
    if (aNum.length != 4) {
        return num;
    }
    num += parseInt(aNum[0]) << 0;
    num += parseInt(aNum[1]) << 8;
    num += parseInt(aNum[2]) << 16;
    num += parseInt(aNum[3]) << 24;
    num = num >>> 0; //这个很关键，不然可能会出现负数的情况
    return num;
}
/**
 * Returns a dictionary of a sockfd's "src_addr", "src_port", "dst_addr", and
 * "dst_port".
 * @param {int} sockfd The file descriptor of the socket to inspect.
 * @param {boolean} isRead If true, the context is an SSL_read call. If
 *     false, the context is an SSL_write call.
 * @return {dict} Dictionary of sockfd's "src_addr", "src_port", "dst_addr",
 *     and "dst_port".
 */
function getPortsAndAddresses(sockfd, isRead) {
    var message = {};
    var src_dst = ["src", "dst"];
    for (var i = 0; i < src_dst.length; i++) {
        if ((src_dst[i] == "src") !== isRead) {
            var sockAddr = Socket.localAddress(sockfd);
        }
        else {
            var sockAddr = Socket.peerAddress(sockfd);
        }
        if (sockAddr == null) {
            // 网络超时or其他原因可能导致socket被关闭
            message[src_dst[i] + "_port"] = 0;
            message[src_dst[i] + "_addr"] = 0;
        }
        else {
            message[src_dst[i] + "_port"] = sockAddr.port & 0xffff;
            message[src_dst[i] + "_addr"] = ntohl(ipToNumber(sockAddr.ip.split(":").pop()));
        }
        console.log("sockfd: " + sockfd + "\tsockAddr: " + sockAddr);
    }
    return message;
}
/**
 * Get the session_id of SSL object and return it as a hex string.
 * @param {!NativePointer} ssl A pointer to an SSL object.
 * @return {dict} A string representing the session_id of the SSL object's
 *     SSL_SESSION. For example,
 *     "59FD71B7B90202F359D89E66AE4E61247954E28431F6C6AC46625D472FF76336".
 */
function getSslSessionId(ssl) {
    var session = SSL_get_session(ssl);
    if (session == 0) {
        return 0;
    }
    var len_ptr = Memory.alloc(4);
    var p = SSL_SESSION_get_id(session, len_ptr);
    console.log("session_id: " + p + "\tsession: " + session + "\tssl: " + ssl);
    var len = len_ptr.readU32();
    var session_id = "";
    console.log("session_id: " + p + "\tsession: " + session + "\tssl: " + ssl + "\tlen: " + len + "data -> " + hexdump(p, { length: len }));
    for (var i = 0; i < len; i++) {
        // Read a byte, convert it to a hex string (0xAB ==> "AB"), and append
        // it to session_id.
        session_id += ("0" + p.add(i).readU8().toString(16).toUpperCase()).substr(-2);
    }
    return session_id;
}
let initialized = false;
function HookSSL(sslwrite_callback, sslread_callback) {
    if (initialized)
        return;
    initialized = true;
    initializeGlobals();
    Interceptor.attach(addresses["SSL_read"], {
        onEnter: function (args) {
            // var message = getPortsAndAddresses(SSL_get_fd!(args[0]), true);
            // message["ssl_session_id"] = getSslSessionId(args[0]);
            // message["function"] = "SSL_read";
            // this.message = message;
            this.ssl = args[0];
            this.buf = args[1];
        },
        onLeave: function (retval) {
            retval |= 0; // Cast retval to 32-bit integer.
            if (retval <= 0) {
                return;
            }
            // console.log(this.buf.readByteArray(retval));
            analyzer.handleRead(this.ssl.toString(), readBuffer(this.buf, Number(retval)), sslread_callback); // Magic
        },
    });
    Interceptor.attach(addresses["SSL_write"], {
        onEnter: function (args) {
            // var message = getPortsAndAddresses(SSL_get_fd!(args[0]), false);
            // message["ssl_session_id"] = getSslSessionId(args[0]);
            // message["function"] = "SSL_write";
            // console.log(args[1].readByteArray(Number(args[2])));
            analyzer.handleWrite(args[0].toString(), readBuffer(args[1], Number(args[2])), sslwrite_callback); // Magic
        },
        onLeave: function (retval) { },
    });
}
function readBuffer(ptr, length) {
    const arrayBuf = ptr.readByteArray(length);
    if (!arrayBuf) {
        return Buffer.alloc(0); // 返回空 Buffer，避免 null 导致崩溃
    }
    return Buffer.from(arrayBuf);
}

}).call(this)}).call(this,require("buffer").Buffer)

},{"./traffic_analyzer":3,"buffer":5}],3:[function(require,module,exports){
(function (Buffer){(function (){
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TrafficAnalyzer = void 0;
/**
 * ------------------------------------------------------------------
 * Connection Context
 * 维护单个 SSL 连接的状态 (Buffer, Protocol, State Machine)
 * ------------------------------------------------------------------
 */
class ConnectionContext {
    sslPtr;
    protocol = "UNKNOWN";
    // 读写分别维护 Buffer（因为全双工）
    readBuffer = Buffer.alloc(0);
    writeBuffer = Buffer.alloc(0);
    // --- HTTP/1.1 暂存区 (等待 Body) ---
    h1PendingReq = null;
    h1PendingResp = null;
    // --- HTTP/2 暂存区 (等待 END_STREAM) ---
    // Map<StreamID, PartialMessage>
    h2StreamStore = new Map();
    constructor(sslPtr) {
        this.sslPtr = sslPtr;
    }
    /**
     * 处理新到达的数据块
     * @param data 新收到的数据
     * @param direction 'READ' (Response) | 'WRITE' (Request)
     * @param onMessage 解析出完整消息时的回调
     */
    onData(data, direction, onMessage) {
        // 1. 追加数据到对应 Buffer
        if (direction === "READ") {
            this.readBuffer = Buffer.concat([this.readBuffer, data]);
        }
        else {
            this.writeBuffer = Buffer.concat([this.writeBuffer, data]);
        }
        // 2. 尝试探测协议（如果未知）
        if (this.protocol === "UNKNOWN") {
            const buf = direction === "READ" ? this.readBuffer : this.writeBuffer;
            this.detectProtocol(buf);
        }
        // 3. 根据协议循环解析
        let buffer = direction === "READ" ? this.readBuffer : this.writeBuffer;
        if (this.protocol === "HTTP/1.1") {
            buffer = this.processHttp1(buffer, direction, onMessage);
        }
        else if (this.protocol === "HTTP/2") {
            buffer = this.processHttp2(buffer, direction, onMessage);
        }
        // 4. 更新回 Context 的 Buffer (BUG FIX)
        if (direction === "READ") {
            this.readBuffer = buffer;
        }
        else {
            this.writeBuffer = buffer;
        }
    }
    detectProtocol(buffer) {
        if (buffer.length < 24)
            return; // 数据太少，等下一次
        const str = buffer.subarray(0, 24).toString();
        // H2 Connection Preface
        if (str.startsWith("PRI * HTTP/2.0")) {
            this.protocol = "HTTP/2";
            console.log(`[${this.sslPtr}] Protocol detected: HTTP/2`);
        }
        else if (str.startsWith("HTTP/") || /^(GET|POST|PUT|HEAD|DELETE)/.test(str)) {
            this.protocol = "HTTP/1.1";
            console.log(`[${this.sslPtr}] Protocol detected: HTTP/1.1`);
        }
    }
    parseH1Headers(str) {
        const lines = str.split("\r\n");
        const headers = {};
        for (let i = 1; i < lines.length; i++) {
            const [key, ...vals] = lines[i].split(":");
            if (key) {
                headers[key.trim()] = vals.join(":").trim();
            }
        }
        return headers;
    }
    /**
     * ------------------------------------------------------------------
     * HTTP/1.1 解析逻辑 (基于 Content-Length 的状态机)
     * ------------------------------------------------------------------
     */
    processHttp1(buffer, direction, callback) {
        // 获取当前方向需要的字节数引用
        let pending = direction === "WRITE" ? this.h1PendingReq : this.h1PendingResp;
        while (buffer.length > 0) {
            // 状态 1: 正在等待 Header (bytesExpected === 0)
            if (!pending) {
                const boundary = buffer.indexOf("\r\n\r\n");
                if (boundary === -1) {
                    break; // Header 还没收全，等待
                }
                // 提取 Header 部分
                const headerBuf = buffer.subarray(0, boundary + 4);
                const headerStr = headerBuf.toString();
                // 提取 Method 和 URL (仅针对 Request)
                let method;
                let url;
                if (direction === "WRITE") {
                    const firstLine = headerStr.split("\r\n")[0];
                    const parts = firstLine.split(" ");
                    if (parts.length >= 2) {
                        method = parts[0];
                        url = parts[1];
                    }
                }
                // 解析 Content-Length
                const clMatch = headerStr.match(/Content-Length:\s*(\d+)/i);
                const contentLength = clMatch ? parseInt(clMatch[1], 10) : 0;
                const headers = this.parseH1Headers(headerStr);
                // 判断是否是 Chunked (这里简化处理，只做标记，不解包 Chunk)
                // TODO: 如果是 Transfer-Encoding: chunked，这里需要更复杂的逻辑。
                // 暂时假设绝大多数 API 都是 Content-Length 或 0
                const isChunked = /Transfer-Encoding:\s*chunked/i.test(headerStr);
                // 建立暂存对象
                pending = { headers, contentLength, method, url };
                // 从 buffer 切掉 Header，准备进入阶段 2
                buffer = buffer.subarray(boundary + 4);
            }
            if (pending) {
                if (buffer.length >= pending.contentLength) {
                    // Body 齐了！切下来
                    // console.log("buffer:", buffer);
                    const body = Buffer.from(buffer.subarray(0, pending.contentLength));
                    // console.log("body:", body);
                    // --- 触发回调 (完整的 Header + Body) ---
                    callback({
                        protocol: "HTTP/1.1",
                        streamId: 1,
                        type: direction === "WRITE" ? "REQUEST" : "RESPONSE",
                        headers: pending.headers,
                        method: pending.method,
                        url: pending.url,
                        body: body,
                    });
                    // 消耗掉 Body，重置 Pending 状态
                    buffer = buffer.subarray(pending.contentLength);
                    pending = null;
                }
                else {
                    // Body 还没齐，跳出循环，等待下一次 onData
                    break;
                }
            }
        }
        if (direction === "WRITE") {
            this.h1PendingReq = pending;
        }
        else {
            this.h1PendingResp = pending;
        }
        return buffer;
    }
    /**
     * ------------------------------------------------------------------
     * HTTP/2 解析逻辑 (二进制帧切分)
     * ------------------------------------------------------------------
     *
     * 解析 HTTP/2 帧头 (前 9 字节)
     * +-----------------------------------------------+
     * |                 Length (24)                   |
     * +---------------+---------------+---------------+
     * |   Type (8)    |   Flags (8)   |
     * +-+-------------+---------------+-------------------------------+
     * |R|                 Stream Identifier (31)                      |
     * +-+-------------------------------------------------------------+
     *
     * todo! unchecked
     */
    processHttp2(buffer, direction, callback) {
        while (true) {
            // 跳过 Preface
            if (buffer.toString().startsWith("PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n")) {
                buffer = buffer.slice(24);
                continue;
            }
            if (buffer.length < 9)
                break; // 帧头不足
            const length = (buffer[0] << 16) | (buffer[1] << 8) | buffer[2];
            const frameSize = 9 + length;
            if (buffer.length < frameSize)
                break; // 帧体不足
            // --- 提取完整帧 ---
            const frame = buffer.slice(0, frameSize);
            const frameType = frame[3];
            const flags = frame[4];
            const streamId = frame.readUInt32BE(5) & 0x7ffffff;
            // 0x1 = HEADERS, 0x0 = DATA
            if (frameType === 0x1 || frameType === 0x0) {
                // 获取或创建流存储
                let streamCtx = this.h2StreamStore.get(streamId);
                if (!streamCtx) {
                    streamCtx = { headers: {}, body: Buffer.alloc(0), type: direction === "WRITE" ? "REQUEST" : "RESPONSE" };
                    this.h2StreamStore.set(streamId, streamCtx);
                }
                // 如果是 DATA 帧，追加 Body
                if (frameType === 0x0) {
                    streamCtx.body = Buffer.concat([streamCtx.body, frame.slice(9)]);
                }
                // 如果是 HEADERS 帧，通常需要 HPACK 解码才能拿到 Header
                // 这里为了演示，我们只标记它收到了 Header 帧
                else if (frameType === 0x1) {
                    // streamCtx.headers = ... (需要 HPACK 解码库)
                }
                // --- 检查 END_STREAM 标志 (Bit 0) ---
                const isEndStream = (flags & 0x1) === 0x1;
                if (isEndStream) {
                    // 流结束了！此时 streamCtx 里有完整的数据
                    callback({
                        protocol: "HTTP/2",
                        streamId: streamId,
                        type: streamCtx.type,
                        headers: streamCtx.headers, // 注意：这里是空的，除非你接入 hpack
                        body: Buffer.from(streamCtx.body),
                    });
                    // 清理内存
                    this.h2StreamStore.delete(streamId);
                }
            }
            buffer = buffer.slice(frameSize);
        }
        return buffer;
    }
}
/**
 * ------------------------------------------------------------------
 * Traffic Analyzer (Main Entry)
 * 管理所有 SSL 连接
 * ------------------------------------------------------------------
 */
class TrafficAnalyzer {
    // Map<SSL指针字符串, 上下文>
    connections = new Map();
    // 新增：为每个 SSL 连接维护一个请求队列
    requestQueues = new Map();
    // 新增：全局请求ID计数器
    nextRequestId = 1;
    /**
     * 模拟 Hook 到 SSL_write (Request)
     */
    handleWrite(sslPtr, data, callback) {
        this.processData(sslPtr, data, "WRITE", callback);
    }
    /**
     * 模拟 Hook 到 SSL_read (Response)
     */
    handleRead(sslPtr, data, callback) {
        this.processData(sslPtr, data, "READ", callback);
    }
    processData(sslPtr, data, dir, callback) {
        let ctx = this.connections.get(sslPtr);
        if (!ctx) {
            ctx = new ConnectionContext(sslPtr);
            this.connections.set(sslPtr, ctx);
        }
        // 回调函数现在将处理配对逻辑
        ctx.onData(data, dir, (msg) => {
            this.pairMessage(sslPtr, msg);
            callback ? callback(msg) : this.logMessage(sslPtr, msg);
        });
    }
    pairMessage(sslPtr, msg) {
        // 获取或创建当前连接的请求队列
        if (!this.requestQueues.has(sslPtr)) {
            this.requestQueues.set(sslPtr, []);
        }
        const queue = this.requestQueues.get(sslPtr);
        if (msg.type === "REQUEST") {
            // 1. 为请求分配ID
            msg.requestId = sslPtr.concat((this.nextRequestId++).toString());
            // 3. 将带ID的请求入队
            queue.push(msg);
        }
        else if (msg.type === "RESPONSE") {
            const request = queue.shift();
            if (request) {
                // 1. 从队列取出请求，将其ID赋给响应
                msg.requestId = request.requestId;
            }
            else {
                // 无法配对的响应
                console.error(`\n[Orphan RESPONSE Captured] on SSL: ${sslPtr}\n${msg}`);
            }
        }
    }
    // 新增的配对和日志记录函数
    logMessage(sslPtr, msg) {
        if (msg.type === "REQUEST") {
            console.log(`\n>>> [REQUEST Captured] ID: ${msg.requestId} on SSL: ${sslPtr}`);
        }
        else if (msg.type === "RESPONSE") {
            console.log(`\n<<< [RESPONSE Captured] For Request ID: ${msg.requestId} on SSL: ${sslPtr}`);
        }
        this.logMessageDetails(msg);
    }
    // logMessage 现在只负责打印单个消息的细节
    logMessageDetails(msg) {
        const idTag = msg.requestId ? `(ID: ${msg.requestId})` : "";
        console.log(`  Type: ${msg.type} ${idTag} | Proto: ${msg.protocol}`);
        if (msg.protocol === "HTTP/1.1") {
            if (msg.method) {
                console.log(`  Request: ${msg.method} ${msg.url}`);
            }
            console.log(`  Headers: ${JSON.stringify(msg.headers)}`);
        }
        else {
            // HTTP/2
            console.log(`  Stream ID: ${msg.streamId}`);
        }
        if (msg.body && msg.body.length > 0) {
            console.log(`  Body Length: ${msg.body.length}`);
            // 为了避免刷屏，只打印 body 的一部分
            const bodyForDisplay = msg.body.toString("utf-8");
            console.log(`  Body Preview: ${bodyForDisplay}`);
        }
        else {
            console.log(`  Body: (empty)`);
        }
    }
}
exports.TrafficAnalyzer = TrafficAnalyzer;

}).call(this)}).call(this,require("buffer").Buffer)

},{"buffer":5}],4:[function(require,module,exports){
'use strict'

exports.byteLength = byteLength
exports.toByteArray = toByteArray
exports.fromByteArray = fromByteArray

var lookup = []
var revLookup = []
var Arr = typeof Uint8Array !== 'undefined' ? Uint8Array : Array

var code = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'
for (var i = 0, len = code.length; i < len; ++i) {
  lookup[i] = code[i]
  revLookup[code.charCodeAt(i)] = i
}

// Support decoding URL-safe base64 strings, as Node.js does.
// See: https://en.wikipedia.org/wiki/Base64#URL_applications
revLookup['-'.charCodeAt(0)] = 62
revLookup['_'.charCodeAt(0)] = 63

function getLens (b64) {
  var len = b64.length

  if (len % 4 > 0) {
    throw new Error('Invalid string. Length must be a multiple of 4')
  }

  // Trim off extra bytes after placeholder bytes are found
  // See: https://github.com/beatgammit/base64-js/issues/42
  var validLen = b64.indexOf('=')
  if (validLen === -1) validLen = len

  var placeHoldersLen = validLen === len
    ? 0
    : 4 - (validLen % 4)

  return [validLen, placeHoldersLen]
}

// base64 is 4/3 + up to two characters of the original data
function byteLength (b64) {
  var lens = getLens(b64)
  var validLen = lens[0]
  var placeHoldersLen = lens[1]
  return ((validLen + placeHoldersLen) * 3 / 4) - placeHoldersLen
}

function _byteLength (b64, validLen, placeHoldersLen) {
  return ((validLen + placeHoldersLen) * 3 / 4) - placeHoldersLen
}

function toByteArray (b64) {
  var tmp
  var lens = getLens(b64)
  var validLen = lens[0]
  var placeHoldersLen = lens[1]

  var arr = new Arr(_byteLength(b64, validLen, placeHoldersLen))

  var curByte = 0

  // if there are placeholders, only get up to the last complete 4 chars
  var len = placeHoldersLen > 0
    ? validLen - 4
    : validLen

  var i
  for (i = 0; i < len; i += 4) {
    tmp =
      (revLookup[b64.charCodeAt(i)] << 18) |
      (revLookup[b64.charCodeAt(i + 1)] << 12) |
      (revLookup[b64.charCodeAt(i + 2)] << 6) |
      revLookup[b64.charCodeAt(i + 3)]
    arr[curByte++] = (tmp >> 16) & 0xFF
    arr[curByte++] = (tmp >> 8) & 0xFF
    arr[curByte++] = tmp & 0xFF
  }

  if (placeHoldersLen === 2) {
    tmp =
      (revLookup[b64.charCodeAt(i)] << 2) |
      (revLookup[b64.charCodeAt(i + 1)] >> 4)
    arr[curByte++] = tmp & 0xFF
  }

  if (placeHoldersLen === 1) {
    tmp =
      (revLookup[b64.charCodeAt(i)] << 10) |
      (revLookup[b64.charCodeAt(i + 1)] << 4) |
      (revLookup[b64.charCodeAt(i + 2)] >> 2)
    arr[curByte++] = (tmp >> 8) & 0xFF
    arr[curByte++] = tmp & 0xFF
  }

  return arr
}

function tripletToBase64 (num) {
  return lookup[num >> 18 & 0x3F] +
    lookup[num >> 12 & 0x3F] +
    lookup[num >> 6 & 0x3F] +
    lookup[num & 0x3F]
}

function encodeChunk (uint8, start, end) {
  var tmp
  var output = []
  for (var i = start; i < end; i += 3) {
    tmp =
      ((uint8[i] << 16) & 0xFF0000) +
      ((uint8[i + 1] << 8) & 0xFF00) +
      (uint8[i + 2] & 0xFF)
    output.push(tripletToBase64(tmp))
  }
  return output.join('')
}

function fromByteArray (uint8) {
  var tmp
  var len = uint8.length
  var extraBytes = len % 3 // if we have 1 byte left, pad 2 bytes
  var parts = []
  var maxChunkLength = 16383 // must be multiple of 3

  // go through the array every three bytes, we'll deal with trailing stuff later
  for (var i = 0, len2 = len - extraBytes; i < len2; i += maxChunkLength) {
    parts.push(encodeChunk(uint8, i, (i + maxChunkLength) > len2 ? len2 : (i + maxChunkLength)))
  }

  // pad the end with zeros, but make sure to not forget the extra bytes
  if (extraBytes === 1) {
    tmp = uint8[len - 1]
    parts.push(
      lookup[tmp >> 2] +
      lookup[(tmp << 4) & 0x3F] +
      '=='
    )
  } else if (extraBytes === 2) {
    tmp = (uint8[len - 2] << 8) + uint8[len - 1]
    parts.push(
      lookup[tmp >> 10] +
      lookup[(tmp >> 4) & 0x3F] +
      lookup[(tmp << 2) & 0x3F] +
      '='
    )
  }

  return parts.join('')
}

},{}],5:[function(require,module,exports){
(function (global){(function (){
/*
 * Short-circuit auto-detection in the buffer module to avoid a Duktape
 * compatibility issue with __proto__.
 */
global.TYPED_ARRAY_SUPPORT = true;

module.exports = require('buffer/');

}).call(this)}).call(this,typeof global !== "undefined" ? global : typeof self !== "undefined" ? self : typeof window !== "undefined" ? window : {})

},{"buffer/":6}],6:[function(require,module,exports){
(function (Buffer){(function (){
/*!
 * The buffer module from node.js, for the browser.
 *
 * @author   Feross Aboukhadijeh <https://feross.org>
 * @license  MIT
 */
/* eslint-disable no-proto */

'use strict'

var base64 = require('base64-js')
var ieee754 = require('ieee754')
var customInspectSymbol =
  (typeof Symbol === 'function' && typeof Symbol['for'] === 'function') // eslint-disable-line dot-notation
    ? Symbol['for']('nodejs.util.inspect.custom') // eslint-disable-line dot-notation
    : null

exports.Buffer = Buffer
exports.SlowBuffer = SlowBuffer
exports.INSPECT_MAX_BYTES = 50

var K_MAX_LENGTH = 0x7fffffff
exports.kMaxLength = K_MAX_LENGTH

/**
 * If `Buffer.TYPED_ARRAY_SUPPORT`:
 *   === true    Use Uint8Array implementation (fastest)
 *   === false   Print warning and recommend using `buffer` v4.x which has an Object
 *               implementation (most compatible, even IE6)
 *
 * Browsers that support typed arrays are IE 10+, Firefox 4+, Chrome 7+, Safari 5.1+,
 * Opera 11.6+, iOS 4.2+.
 *
 * We report that the browser does not support typed arrays if the are not subclassable
 * using __proto__. Firefox 4-29 lacks support for adding new properties to `Uint8Array`
 * (See: https://bugzilla.mozilla.org/show_bug.cgi?id=695438). IE 10 lacks support
 * for __proto__ and has a buggy typed array implementation.
 */
Buffer.TYPED_ARRAY_SUPPORT = typedArraySupport()

if (!Buffer.TYPED_ARRAY_SUPPORT && typeof console !== 'undefined' &&
    typeof console.error === 'function') {
  console.error(
    'This browser lacks typed array (Uint8Array) support which is required by ' +
    '`buffer` v5.x. Use `buffer` v4.x if you require old browser support.'
  )
}

function typedArraySupport () {
  // Can typed array instances can be augmented?
  try {
    var arr = new Uint8Array(1)
    var proto = { foo: function () { return 42 } }
    Object.setPrototypeOf(proto, Uint8Array.prototype)
    Object.setPrototypeOf(arr, proto)
    return arr.foo() === 42
  } catch (e) {
    return false
  }
}

Object.defineProperty(Buffer.prototype, 'parent', {
  enumerable: true,
  get: function () {
    if (!Buffer.isBuffer(this)) return undefined
    return this.buffer
  }
})

Object.defineProperty(Buffer.prototype, 'offset', {
  enumerable: true,
  get: function () {
    if (!Buffer.isBuffer(this)) return undefined
    return this.byteOffset
  }
})

function createBuffer (length) {
  if (length > K_MAX_LENGTH) {
    throw new RangeError('The value "' + length + '" is invalid for option "size"')
  }
  // Return an augmented `Uint8Array` instance
  var buf = new Uint8Array(length)
  Object.setPrototypeOf(buf, Buffer.prototype)
  return buf
}

/**
 * The Buffer constructor returns instances of `Uint8Array` that have their
 * prototype changed to `Buffer.prototype`. Furthermore, `Buffer` is a subclass of
 * `Uint8Array`, so the returned instances will have all the node `Buffer` methods
 * and the `Uint8Array` methods. Square bracket notation works as expected -- it
 * returns a single octet.
 *
 * The `Uint8Array` prototype remains unmodified.
 */

function Buffer (arg, encodingOrOffset, length) {
  // Common case.
  if (typeof arg === 'number') {
    if (typeof encodingOrOffset === 'string') {
      throw new TypeError(
        'The "string" argument must be of type string. Received type number'
      )
    }
    return allocUnsafe(arg)
  }
  return from(arg, encodingOrOffset, length)
}

Buffer.poolSize = 8192 // not used by this implementation

function from (value, encodingOrOffset, length) {
  if (typeof value === 'string') {
    return fromString(value, encodingOrOffset)
  }

  if (ArrayBuffer.isView(value)) {
    return fromArrayView(value)
  }

  if (value == null) {
    throw new TypeError(
      'The first argument must be one of type string, Buffer, ArrayBuffer, Array, ' +
      'or Array-like Object. Received type ' + (typeof value)
    )
  }

  if (isInstance(value, ArrayBuffer) ||
      (value && isInstance(value.buffer, ArrayBuffer))) {
    return fromArrayBuffer(value, encodingOrOffset, length)
  }

  if (typeof SharedArrayBuffer !== 'undefined' &&
      (isInstance(value, SharedArrayBuffer) ||
      (value && isInstance(value.buffer, SharedArrayBuffer)))) {
    return fromArrayBuffer(value, encodingOrOffset, length)
  }

  if (typeof value === 'number') {
    throw new TypeError(
      'The "value" argument must not be of type number. Received type number'
    )
  }

  var valueOf = value.valueOf && value.valueOf()
  if (valueOf != null && valueOf !== value) {
    return Buffer.from(valueOf, encodingOrOffset, length)
  }

  var b = fromObject(value)
  if (b) return b

  if (typeof Symbol !== 'undefined' && Symbol.toPrimitive != null &&
      typeof value[Symbol.toPrimitive] === 'function') {
    return Buffer.from(
      value[Symbol.toPrimitive]('string'), encodingOrOffset, length
    )
  }

  throw new TypeError(
    'The first argument must be one of type string, Buffer, ArrayBuffer, Array, ' +
    'or Array-like Object. Received type ' + (typeof value)
  )
}

/**
 * Functionally equivalent to Buffer(arg, encoding) but throws a TypeError
 * if value is a number.
 * Buffer.from(str[, encoding])
 * Buffer.from(array)
 * Buffer.from(buffer)
 * Buffer.from(arrayBuffer[, byteOffset[, length]])
 **/
Buffer.from = function (value, encodingOrOffset, length) {
  return from(value, encodingOrOffset, length)
}

// Note: Change prototype *after* Buffer.from is defined to workaround Chrome bug:
// https://github.com/feross/buffer/pull/148
Object.setPrototypeOf(Buffer.prototype, Uint8Array.prototype)
Object.setPrototypeOf(Buffer, Uint8Array)

function assertSize (size) {
  if (typeof size !== 'number') {
    throw new TypeError('"size" argument must be of type number')
  } else if (size < 0) {
    throw new RangeError('The value "' + size + '" is invalid for option "size"')
  }
}

function alloc (size, fill, encoding) {
  assertSize(size)
  if (size <= 0) {
    return createBuffer(size)
  }
  if (fill !== undefined) {
    // Only pay attention to encoding if it's a string. This
    // prevents accidentally sending in a number that would
    // be interpreted as a start offset.
    return typeof encoding === 'string'
      ? createBuffer(size).fill(fill, encoding)
      : createBuffer(size).fill(fill)
  }
  return createBuffer(size)
}

/**
 * Creates a new filled Buffer instance.
 * alloc(size[, fill[, encoding]])
 **/
Buffer.alloc = function (size, fill, encoding) {
  return alloc(size, fill, encoding)
}

function allocUnsafe (size) {
  assertSize(size)
  return createBuffer(size < 0 ? 0 : checked(size) | 0)
}

/**
 * Equivalent to Buffer(num), by default creates a non-zero-filled Buffer instance.
 * */
Buffer.allocUnsafe = function (size) {
  return allocUnsafe(size)
}
/**
 * Equivalent to SlowBuffer(num), by default creates a non-zero-filled Buffer instance.
 */
Buffer.allocUnsafeSlow = function (size) {
  return allocUnsafe(size)
}

function fromString (string, encoding) {
  if (typeof encoding !== 'string' || encoding === '') {
    encoding = 'utf8'
  }

  if (!Buffer.isEncoding(encoding)) {
    throw new TypeError('Unknown encoding: ' + encoding)
  }

  var length = byteLength(string, encoding) | 0
  var buf = createBuffer(length)

  var actual = buf.write(string, encoding)

  if (actual !== length) {
    // Writing a hex string, for example, that contains invalid characters will
    // cause everything after the first invalid character to be ignored. (e.g.
    // 'abxxcd' will be treated as 'ab')
    buf = buf.slice(0, actual)
  }

  return buf
}

function fromArrayLike (array) {
  var length = array.length < 0 ? 0 : checked(array.length) | 0
  var buf = createBuffer(length)
  for (var i = 0; i < length; i += 1) {
    buf[i] = array[i] & 255
  }
  return buf
}

function fromArrayView (arrayView) {
  if (isInstance(arrayView, Uint8Array)) {
    var copy = new Uint8Array(arrayView)
    return fromArrayBuffer(copy.buffer, copy.byteOffset, copy.byteLength)
  }
  return fromArrayLike(arrayView)
}

function fromArrayBuffer (array, byteOffset, length) {
  if (byteOffset < 0 || array.byteLength < byteOffset) {
    throw new RangeError('"offset" is outside of buffer bounds')
  }

  if (array.byteLength < byteOffset + (length || 0)) {
    throw new RangeError('"length" is outside of buffer bounds')
  }

  var buf
  if (byteOffset === undefined && length === undefined) {
    buf = new Uint8Array(array)
  } else if (length === undefined) {
    buf = new Uint8Array(array, byteOffset)
  } else {
    buf = new Uint8Array(array, byteOffset, length)
  }

  // Return an augmented `Uint8Array` instance
  Object.setPrototypeOf(buf, Buffer.prototype)

  return buf
}

function fromObject (obj) {
  if (Buffer.isBuffer(obj)) {
    var len = checked(obj.length) | 0
    var buf = createBuffer(len)

    if (buf.length === 0) {
      return buf
    }

    obj.copy(buf, 0, 0, len)
    return buf
  }

  if (obj.length !== undefined) {
    if (typeof obj.length !== 'number' || numberIsNaN(obj.length)) {
      return createBuffer(0)
    }
    return fromArrayLike(obj)
  }

  if (obj.type === 'Buffer' && Array.isArray(obj.data)) {
    return fromArrayLike(obj.data)
  }
}

function checked (length) {
  // Note: cannot use `length < K_MAX_LENGTH` here because that fails when
  // length is NaN (which is otherwise coerced to zero.)
  if (length >= K_MAX_LENGTH) {
    throw new RangeError('Attempt to allocate Buffer larger than maximum ' +
                         'size: 0x' + K_MAX_LENGTH.toString(16) + ' bytes')
  }
  return length | 0
}

function SlowBuffer (length) {
  if (+length != length) { // eslint-disable-line eqeqeq
    length = 0
  }
  return Buffer.alloc(+length)
}

Buffer.isBuffer = function isBuffer (b) {
  return b != null && b._isBuffer === true &&
    b !== Buffer.prototype // so Buffer.isBuffer(Buffer.prototype) will be false
}

Buffer.compare = function compare (a, b) {
  if (isInstance(a, Uint8Array)) a = Buffer.from(a, a.offset, a.byteLength)
  if (isInstance(b, Uint8Array)) b = Buffer.from(b, b.offset, b.byteLength)
  if (!Buffer.isBuffer(a) || !Buffer.isBuffer(b)) {
    throw new TypeError(
      'The "buf1", "buf2" arguments must be one of type Buffer or Uint8Array'
    )
  }

  if (a === b) return 0

  var x = a.length
  var y = b.length

  for (var i = 0, len = Math.min(x, y); i < len; ++i) {
    if (a[i] !== b[i]) {
      x = a[i]
      y = b[i]
      break
    }
  }

  if (x < y) return -1
  if (y < x) return 1
  return 0
}

Buffer.isEncoding = function isEncoding (encoding) {
  switch (String(encoding).toLowerCase()) {
    case 'hex':
    case 'utf8':
    case 'utf-8':
    case 'ascii':
    case 'latin1':
    case 'binary':
    case 'base64':
    case 'ucs2':
    case 'ucs-2':
    case 'utf16le':
    case 'utf-16le':
      return true
    default:
      return false
  }
}

Buffer.concat = function concat (list, length) {
  if (!Array.isArray(list)) {
    throw new TypeError('"list" argument must be an Array of Buffers')
  }

  if (list.length === 0) {
    return Buffer.alloc(0)
  }

  var i
  if (length === undefined) {
    length = 0
    for (i = 0; i < list.length; ++i) {
      length += list[i].length
    }
  }

  var buffer = Buffer.allocUnsafe(length)
  var pos = 0
  for (i = 0; i < list.length; ++i) {
    var buf = list[i]
    if (isInstance(buf, Uint8Array)) {
      if (pos + buf.length > buffer.length) {
        Buffer.from(buf).copy(buffer, pos)
      } else {
        Uint8Array.prototype.set.call(
          buffer,
          buf,
          pos
        )
      }
    } else if (!Buffer.isBuffer(buf)) {
      throw new TypeError('"list" argument must be an Array of Buffers')
    } else {
      buf.copy(buffer, pos)
    }
    pos += buf.length
  }
  return buffer
}

function byteLength (string, encoding) {
  if (Buffer.isBuffer(string)) {
    return string.length
  }
  if (ArrayBuffer.isView(string) || isInstance(string, ArrayBuffer)) {
    return string.byteLength
  }
  if (typeof string !== 'string') {
    throw new TypeError(
      'The "string" argument must be one of type string, Buffer, or ArrayBuffer. ' +
      'Received type ' + typeof string
    )
  }

  var len = string.length
  var mustMatch = (arguments.length > 2 && arguments[2] === true)
  if (!mustMatch && len === 0) return 0

  // Use a for loop to avoid recursion
  var loweredCase = false
  for (;;) {
    switch (encoding) {
      case 'ascii':
      case 'latin1':
      case 'binary':
        return len
      case 'utf8':
      case 'utf-8':
        return utf8ToBytes(string).length
      case 'ucs2':
      case 'ucs-2':
      case 'utf16le':
      case 'utf-16le':
        return len * 2
      case 'hex':
        return len >>> 1
      case 'base64':
        return base64ToBytes(string).length
      default:
        if (loweredCase) {
          return mustMatch ? -1 : utf8ToBytes(string).length // assume utf8
        }
        encoding = ('' + encoding).toLowerCase()
        loweredCase = true
    }
  }
}
Buffer.byteLength = byteLength

function slowToString (encoding, start, end) {
  var loweredCase = false

  // No need to verify that "this.length <= MAX_UINT32" since it's a read-only
  // property of a typed array.

  // This behaves neither like String nor Uint8Array in that we set start/end
  // to their upper/lower bounds if the value passed is out of range.
  // undefined is handled specially as per ECMA-262 6th Edition,
  // Section 13.3.3.7 Runtime Semantics: KeyedBindingInitialization.
  if (start === undefined || start < 0) {
    start = 0
  }
  // Return early if start > this.length. Done here to prevent potential uint32
  // coercion fail below.
  if (start > this.length) {
    return ''
  }

  if (end === undefined || end > this.length) {
    end = this.length
  }

  if (end <= 0) {
    return ''
  }

  // Force coercion to uint32. This will also coerce falsey/NaN values to 0.
  end >>>= 0
  start >>>= 0

  if (end <= start) {
    return ''
  }

  if (!encoding) encoding = 'utf8'

  while (true) {
    switch (encoding) {
      case 'hex':
        return hexSlice(this, start, end)

      case 'utf8':
      case 'utf-8':
        return utf8Slice(this, start, end)

      case 'ascii':
        return asciiSlice(this, start, end)

      case 'latin1':
      case 'binary':
        return latin1Slice(this, start, end)

      case 'base64':
        return base64Slice(this, start, end)

      case 'ucs2':
      case 'ucs-2':
      case 'utf16le':
      case 'utf-16le':
        return utf16leSlice(this, start, end)

      default:
        if (loweredCase) throw new TypeError('Unknown encoding: ' + encoding)
        encoding = (encoding + '').toLowerCase()
        loweredCase = true
    }
  }
}

// This property is used by `Buffer.isBuffer` (and the `is-buffer` npm package)
// to detect a Buffer instance. It's not possible to use `instanceof Buffer`
// reliably in a browserify context because there could be multiple different
// copies of the 'buffer' package in use. This method works even for Buffer
// instances that were created from another copy of the `buffer` package.
// See: https://github.com/feross/buffer/issues/154
Buffer.prototype._isBuffer = true

function swap (b, n, m) {
  var i = b[n]
  b[n] = b[m]
  b[m] = i
}

Buffer.prototype.swap16 = function swap16 () {
  var len = this.length
  if (len % 2 !== 0) {
    throw new RangeError('Buffer size must be a multiple of 16-bits')
  }
  for (var i = 0; i < len; i += 2) {
    swap(this, i, i + 1)
  }
  return this
}

Buffer.prototype.swap32 = function swap32 () {
  var len = this.length
  if (len % 4 !== 0) {
    throw new RangeError('Buffer size must be a multiple of 32-bits')
  }
  for (var i = 0; i < len; i += 4) {
    swap(this, i, i + 3)
    swap(this, i + 1, i + 2)
  }
  return this
}

Buffer.prototype.swap64 = function swap64 () {
  var len = this.length
  if (len % 8 !== 0) {
    throw new RangeError('Buffer size must be a multiple of 64-bits')
  }
  for (var i = 0; i < len; i += 8) {
    swap(this, i, i + 7)
    swap(this, i + 1, i + 6)
    swap(this, i + 2, i + 5)
    swap(this, i + 3, i + 4)
  }
  return this
}

Buffer.prototype.toString = function toString () {
  var length = this.length
  if (length === 0) return ''
  if (arguments.length === 0) return utf8Slice(this, 0, length)
  return slowToString.apply(this, arguments)
}

Buffer.prototype.toLocaleString = Buffer.prototype.toString

Buffer.prototype.equals = function equals (b) {
  if (!Buffer.isBuffer(b)) throw new TypeError('Argument must be a Buffer')
  if (this === b) return true
  return Buffer.compare(this, b) === 0
}

Buffer.prototype.inspect = function inspect () {
  var str = ''
  var max = exports.INSPECT_MAX_BYTES
  str = this.toString('hex', 0, max).replace(/(.{2})/g, '$1 ').trim()
  if (this.length > max) str += ' ... '
  return '<Buffer ' + str + '>'
}
if (customInspectSymbol) {
  Buffer.prototype[customInspectSymbol] = Buffer.prototype.inspect
}

Buffer.prototype.compare = function compare (target, start, end, thisStart, thisEnd) {
  if (isInstance(target, Uint8Array)) {
    target = Buffer.from(target, target.offset, target.byteLength)
  }
  if (!Buffer.isBuffer(target)) {
    throw new TypeError(
      'The "target" argument must be one of type Buffer or Uint8Array. ' +
      'Received type ' + (typeof target)
    )
  }

  if (start === undefined) {
    start = 0
  }
  if (end === undefined) {
    end = target ? target.length : 0
  }
  if (thisStart === undefined) {
    thisStart = 0
  }
  if (thisEnd === undefined) {
    thisEnd = this.length
  }

  if (start < 0 || end > target.length || thisStart < 0 || thisEnd > this.length) {
    throw new RangeError('out of range index')
  }

  if (thisStart >= thisEnd && start >= end) {
    return 0
  }
  if (thisStart >= thisEnd) {
    return -1
  }
  if (start >= end) {
    return 1
  }

  start >>>= 0
  end >>>= 0
  thisStart >>>= 0
  thisEnd >>>= 0

  if (this === target) return 0

  var x = thisEnd - thisStart
  var y = end - start
  var len = Math.min(x, y)

  var thisCopy = this.slice(thisStart, thisEnd)
  var targetCopy = target.slice(start, end)

  for (var i = 0; i < len; ++i) {
    if (thisCopy[i] !== targetCopy[i]) {
      x = thisCopy[i]
      y = targetCopy[i]
      break
    }
  }

  if (x < y) return -1
  if (y < x) return 1
  return 0
}

// Finds either the first index of `val` in `buffer` at offset >= `byteOffset`,
// OR the last index of `val` in `buffer` at offset <= `byteOffset`.
//
// Arguments:
// - buffer - a Buffer to search
// - val - a string, Buffer, or number
// - byteOffset - an index into `buffer`; will be clamped to an int32
// - encoding - an optional encoding, relevant is val is a string
// - dir - true for indexOf, false for lastIndexOf
function bidirectionalIndexOf (buffer, val, byteOffset, encoding, dir) {
  // Empty buffer means no match
  if (buffer.length === 0) return -1

  // Normalize byteOffset
  if (typeof byteOffset === 'string') {
    encoding = byteOffset
    byteOffset = 0
  } else if (byteOffset > 0x7fffffff) {
    byteOffset = 0x7fffffff
  } else if (byteOffset < -0x80000000) {
    byteOffset = -0x80000000
  }
  byteOffset = +byteOffset // Coerce to Number.
  if (numberIsNaN(byteOffset)) {
    // byteOffset: it it's undefined, null, NaN, "foo", etc, search whole buffer
    byteOffset = dir ? 0 : (buffer.length - 1)
  }

  // Normalize byteOffset: negative offsets start from the end of the buffer
  if (byteOffset < 0) byteOffset = buffer.length + byteOffset
  if (byteOffset >= buffer.length) {
    if (dir) return -1
    else byteOffset = buffer.length - 1
  } else if (byteOffset < 0) {
    if (dir) byteOffset = 0
    else return -1
  }

  // Normalize val
  if (typeof val === 'string') {
    val = Buffer.from(val, encoding)
  }

  // Finally, search either indexOf (if dir is true) or lastIndexOf
  if (Buffer.isBuffer(val)) {
    // Special case: looking for empty string/buffer always fails
    if (val.length === 0) {
      return -1
    }
    return arrayIndexOf(buffer, val, byteOffset, encoding, dir)
  } else if (typeof val === 'number') {
    val = val & 0xFF // Search for a byte value [0-255]
    if (typeof Uint8Array.prototype.indexOf === 'function') {
      if (dir) {
        return Uint8Array.prototype.indexOf.call(buffer, val, byteOffset)
      } else {
        return Uint8Array.prototype.lastIndexOf.call(buffer, val, byteOffset)
      }
    }
    return arrayIndexOf(buffer, [val], byteOffset, encoding, dir)
  }

  throw new TypeError('val must be string, number or Buffer')
}

function arrayIndexOf (arr, val, byteOffset, encoding, dir) {
  var indexSize = 1
  var arrLength = arr.length
  var valLength = val.length

  if (encoding !== undefined) {
    encoding = String(encoding).toLowerCase()
    if (encoding === 'ucs2' || encoding === 'ucs-2' ||
        encoding === 'utf16le' || encoding === 'utf-16le') {
      if (arr.length < 2 || val.length < 2) {
        return -1
      }
      indexSize = 2
      arrLength /= 2
      valLength /= 2
      byteOffset /= 2
    }
  }

  function read (buf, i) {
    if (indexSize === 1) {
      return buf[i]
    } else {
      return buf.readUInt16BE(i * indexSize)
    }
  }

  var i
  if (dir) {
    var foundIndex = -1
    for (i = byteOffset; i < arrLength; i++) {
      if (read(arr, i) === read(val, foundIndex === -1 ? 0 : i - foundIndex)) {
        if (foundIndex === -1) foundIndex = i
        if (i - foundIndex + 1 === valLength) return foundIndex * indexSize
      } else {
        if (foundIndex !== -1) i -= i - foundIndex
        foundIndex = -1
      }
    }
  } else {
    if (byteOffset + valLength > arrLength) byteOffset = arrLength - valLength
    for (i = byteOffset; i >= 0; i--) {
      var found = true
      for (var j = 0; j < valLength; j++) {
        if (read(arr, i + j) !== read(val, j)) {
          found = false
          break
        }
      }
      if (found) return i
    }
  }

  return -1
}

Buffer.prototype.includes = function includes (val, byteOffset, encoding) {
  return this.indexOf(val, byteOffset, encoding) !== -1
}

Buffer.prototype.indexOf = function indexOf (val, byteOffset, encoding) {
  return bidirectionalIndexOf(this, val, byteOffset, encoding, true)
}

Buffer.prototype.lastIndexOf = function lastIndexOf (val, byteOffset, encoding) {
  return bidirectionalIndexOf(this, val, byteOffset, encoding, false)
}

function hexWrite (buf, string, offset, length) {
  offset = Number(offset) || 0
  var remaining = buf.length - offset
  if (!length) {
    length = remaining
  } else {
    length = Number(length)
    if (length > remaining) {
      length = remaining
    }
  }

  var strLen = string.length

  if (length > strLen / 2) {
    length = strLen / 2
  }
  for (var i = 0; i < length; ++i) {
    var parsed = parseInt(string.substr(i * 2, 2), 16)
    if (numberIsNaN(parsed)) return i
    buf[offset + i] = parsed
  }
  return i
}

function utf8Write (buf, string, offset, length) {
  return blitBuffer(utf8ToBytes(string, buf.length - offset), buf, offset, length)
}

function asciiWrite (buf, string, offset, length) {
  return blitBuffer(asciiToBytes(string), buf, offset, length)
}

function base64Write (buf, string, offset, length) {
  return blitBuffer(base64ToBytes(string), buf, offset, length)
}

function ucs2Write (buf, string, offset, length) {
  return blitBuffer(utf16leToBytes(string, buf.length - offset), buf, offset, length)
}

Buffer.prototype.write = function write (string, offset, length, encoding) {
  // Buffer#write(string)
  if (offset === undefined) {
    encoding = 'utf8'
    length = this.length
    offset = 0
  // Buffer#write(string, encoding)
  } else if (length === undefined && typeof offset === 'string') {
    encoding = offset
    length = this.length
    offset = 0
  // Buffer#write(string, offset[, length][, encoding])
  } else if (isFinite(offset)) {
    offset = offset >>> 0
    if (isFinite(length)) {
      length = length >>> 0
      if (encoding === undefined) encoding = 'utf8'
    } else {
      encoding = length
      length = undefined
    }
  } else {
    throw new Error(
      'Buffer.write(string, encoding, offset[, length]) is no longer supported'
    )
  }

  var remaining = this.length - offset
  if (length === undefined || length > remaining) length = remaining

  if ((string.length > 0 && (length < 0 || offset < 0)) || offset > this.length) {
    throw new RangeError('Attempt to write outside buffer bounds')
  }

  if (!encoding) encoding = 'utf8'

  var loweredCase = false
  for (;;) {
    switch (encoding) {
      case 'hex':
        return hexWrite(this, string, offset, length)

      case 'utf8':
      case 'utf-8':
        return utf8Write(this, string, offset, length)

      case 'ascii':
      case 'latin1':
      case 'binary':
        return asciiWrite(this, string, offset, length)

      case 'base64':
        // Warning: maxLength not taken into account in base64Write
        return base64Write(this, string, offset, length)

      case 'ucs2':
      case 'ucs-2':
      case 'utf16le':
      case 'utf-16le':
        return ucs2Write(this, string, offset, length)

      default:
        if (loweredCase) throw new TypeError('Unknown encoding: ' + encoding)
        encoding = ('' + encoding).toLowerCase()
        loweredCase = true
    }
  }
}

Buffer.prototype.toJSON = function toJSON () {
  return {
    type: 'Buffer',
    data: Array.prototype.slice.call(this._arr || this, 0)
  }
}

function base64Slice (buf, start, end) {
  if (start === 0 && end === buf.length) {
    return base64.fromByteArray(buf)
  } else {
    return base64.fromByteArray(buf.slice(start, end))
  }
}

function utf8Slice (buf, start, end) {
  end = Math.min(buf.length, end)
  var res = []

  var i = start
  while (i < end) {
    var firstByte = buf[i]
    var codePoint = null
    var bytesPerSequence = (firstByte > 0xEF)
      ? 4
      : (firstByte > 0xDF)
          ? 3
          : (firstByte > 0xBF)
              ? 2
              : 1

    if (i + bytesPerSequence <= end) {
      var secondByte, thirdByte, fourthByte, tempCodePoint

      switch (bytesPerSequence) {
        case 1:
          if (firstByte < 0x80) {
            codePoint = firstByte
          }
          break
        case 2:
          secondByte = buf[i + 1]
          if ((secondByte & 0xC0) === 0x80) {
            tempCodePoint = (firstByte & 0x1F) << 0x6 | (secondByte & 0x3F)
            if (tempCodePoint > 0x7F) {
              codePoint = tempCodePoint
            }
          }
          break
        case 3:
          secondByte = buf[i + 1]
          thirdByte = buf[i + 2]
          if ((secondByte & 0xC0) === 0x80 && (thirdByte & 0xC0) === 0x80) {
            tempCodePoint = (firstByte & 0xF) << 0xC | (secondByte & 0x3F) << 0x6 | (thirdByte & 0x3F)
            if (tempCodePoint > 0x7FF && (tempCodePoint < 0xD800 || tempCodePoint > 0xDFFF)) {
              codePoint = tempCodePoint
            }
          }
          break
        case 4:
          secondByte = buf[i + 1]
          thirdByte = buf[i + 2]
          fourthByte = buf[i + 3]
          if ((secondByte & 0xC0) === 0x80 && (thirdByte & 0xC0) === 0x80 && (fourthByte & 0xC0) === 0x80) {
            tempCodePoint = (firstByte & 0xF) << 0x12 | (secondByte & 0x3F) << 0xC | (thirdByte & 0x3F) << 0x6 | (fourthByte & 0x3F)
            if (tempCodePoint > 0xFFFF && tempCodePoint < 0x110000) {
              codePoint = tempCodePoint
            }
          }
      }
    }

    if (codePoint === null) {
      // we did not generate a valid codePoint so insert a
      // replacement char (U+FFFD) and advance only 1 byte
      codePoint = 0xFFFD
      bytesPerSequence = 1
    } else if (codePoint > 0xFFFF) {
      // encode to utf16 (surrogate pair dance)
      codePoint -= 0x10000
      res.push(codePoint >>> 10 & 0x3FF | 0xD800)
      codePoint = 0xDC00 | codePoint & 0x3FF
    }

    res.push(codePoint)
    i += bytesPerSequence
  }

  return decodeCodePointsArray(res)
}

// Based on http://stackoverflow.com/a/22747272/680742, the browser with
// the lowest limit is Chrome, with 0x10000 args.
// We go 1 magnitude less, for safety
var MAX_ARGUMENTS_LENGTH = 0x1000

function decodeCodePointsArray (codePoints) {
  var len = codePoints.length
  if (len <= MAX_ARGUMENTS_LENGTH) {
    return String.fromCharCode.apply(String, codePoints) // avoid extra slice()
  }

  // Decode in chunks to avoid "call stack size exceeded".
  var res = ''
  var i = 0
  while (i < len) {
    res += String.fromCharCode.apply(
      String,
      codePoints.slice(i, i += MAX_ARGUMENTS_LENGTH)
    )
  }
  return res
}

function asciiSlice (buf, start, end) {
  var ret = ''
  end = Math.min(buf.length, end)

  for (var i = start; i < end; ++i) {
    ret += String.fromCharCode(buf[i] & 0x7F)
  }
  return ret
}

function latin1Slice (buf, start, end) {
  var ret = ''
  end = Math.min(buf.length, end)

  for (var i = start; i < end; ++i) {
    ret += String.fromCharCode(buf[i])
  }
  return ret
}

function hexSlice (buf, start, end) {
  var len = buf.length

  if (!start || start < 0) start = 0
  if (!end || end < 0 || end > len) end = len

  var out = ''
  for (var i = start; i < end; ++i) {
    out += hexSliceLookupTable[buf[i]]
  }
  return out
}

function utf16leSlice (buf, start, end) {
  var bytes = buf.slice(start, end)
  var res = ''
  // If bytes.length is odd, the last 8 bits must be ignored (same as node.js)
  for (var i = 0; i < bytes.length - 1; i += 2) {
    res += String.fromCharCode(bytes[i] + (bytes[i + 1] * 256))
  }
  return res
}

Buffer.prototype.slice = function slice (start, end) {
  var len = this.length
  start = ~~start
  end = end === undefined ? len : ~~end

  if (start < 0) {
    start += len
    if (start < 0) start = 0
  } else if (start > len) {
    start = len
  }

  if (end < 0) {
    end += len
    if (end < 0) end = 0
  } else if (end > len) {
    end = len
  }

  if (end < start) end = start

  var newBuf = this.subarray(start, end)
  // Return an augmented `Uint8Array` instance
  Object.setPrototypeOf(newBuf, Buffer.prototype)

  return newBuf
}

/*
 * Need to make sure that buffer isn't trying to write out of bounds.
 */
function checkOffset (offset, ext, length) {
  if ((offset % 1) !== 0 || offset < 0) throw new RangeError('offset is not uint')
  if (offset + ext > length) throw new RangeError('Trying to access beyond buffer length')
}

Buffer.prototype.readUintLE =
Buffer.prototype.readUIntLE = function readUIntLE (offset, byteLength, noAssert) {
  offset = offset >>> 0
  byteLength = byteLength >>> 0
  if (!noAssert) checkOffset(offset, byteLength, this.length)

  var val = this[offset]
  var mul = 1
  var i = 0
  while (++i < byteLength && (mul *= 0x100)) {
    val += this[offset + i] * mul
  }

  return val
}

Buffer.prototype.readUintBE =
Buffer.prototype.readUIntBE = function readUIntBE (offset, byteLength, noAssert) {
  offset = offset >>> 0
  byteLength = byteLength >>> 0
  if (!noAssert) {
    checkOffset(offset, byteLength, this.length)
  }

  var val = this[offset + --byteLength]
  var mul = 1
  while (byteLength > 0 && (mul *= 0x100)) {
    val += this[offset + --byteLength] * mul
  }

  return val
}

Buffer.prototype.readUint8 =
Buffer.prototype.readUInt8 = function readUInt8 (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 1, this.length)
  return this[offset]
}

Buffer.prototype.readUint16LE =
Buffer.prototype.readUInt16LE = function readUInt16LE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 2, this.length)
  return this[offset] | (this[offset + 1] << 8)
}

Buffer.prototype.readUint16BE =
Buffer.prototype.readUInt16BE = function readUInt16BE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 2, this.length)
  return (this[offset] << 8) | this[offset + 1]
}

Buffer.prototype.readUint32LE =
Buffer.prototype.readUInt32LE = function readUInt32LE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 4, this.length)

  return ((this[offset]) |
      (this[offset + 1] << 8) |
      (this[offset + 2] << 16)) +
      (this[offset + 3] * 0x1000000)
}

Buffer.prototype.readUint32BE =
Buffer.prototype.readUInt32BE = function readUInt32BE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 4, this.length)

  return (this[offset] * 0x1000000) +
    ((this[offset + 1] << 16) |
    (this[offset + 2] << 8) |
    this[offset + 3])
}

Buffer.prototype.readIntLE = function readIntLE (offset, byteLength, noAssert) {
  offset = offset >>> 0
  byteLength = byteLength >>> 0
  if (!noAssert) checkOffset(offset, byteLength, this.length)

  var val = this[offset]
  var mul = 1
  var i = 0
  while (++i < byteLength && (mul *= 0x100)) {
    val += this[offset + i] * mul
  }
  mul *= 0x80

  if (val >= mul) val -= Math.pow(2, 8 * byteLength)

  return val
}

Buffer.prototype.readIntBE = function readIntBE (offset, byteLength, noAssert) {
  offset = offset >>> 0
  byteLength = byteLength >>> 0
  if (!noAssert) checkOffset(offset, byteLength, this.length)

  var i = byteLength
  var mul = 1
  var val = this[offset + --i]
  while (i > 0 && (mul *= 0x100)) {
    val += this[offset + --i] * mul
  }
  mul *= 0x80

  if (val >= mul) val -= Math.pow(2, 8 * byteLength)

  return val
}

Buffer.prototype.readInt8 = function readInt8 (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 1, this.length)
  if (!(this[offset] & 0x80)) return (this[offset])
  return ((0xff - this[offset] + 1) * -1)
}

Buffer.prototype.readInt16LE = function readInt16LE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 2, this.length)
  var val = this[offset] | (this[offset + 1] << 8)
  return (val & 0x8000) ? val | 0xFFFF0000 : val
}

Buffer.prototype.readInt16BE = function readInt16BE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 2, this.length)
  var val = this[offset + 1] | (this[offset] << 8)
  return (val & 0x8000) ? val | 0xFFFF0000 : val
}

Buffer.prototype.readInt32LE = function readInt32LE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 4, this.length)

  return (this[offset]) |
    (this[offset + 1] << 8) |
    (this[offset + 2] << 16) |
    (this[offset + 3] << 24)
}

Buffer.prototype.readInt32BE = function readInt32BE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 4, this.length)

  return (this[offset] << 24) |
    (this[offset + 1] << 16) |
    (this[offset + 2] << 8) |
    (this[offset + 3])
}

Buffer.prototype.readFloatLE = function readFloatLE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 4, this.length)
  return ieee754.read(this, offset, true, 23, 4)
}

Buffer.prototype.readFloatBE = function readFloatBE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 4, this.length)
  return ieee754.read(this, offset, false, 23, 4)
}

Buffer.prototype.readDoubleLE = function readDoubleLE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 8, this.length)
  return ieee754.read(this, offset, true, 52, 8)
}

Buffer.prototype.readDoubleBE = function readDoubleBE (offset, noAssert) {
  offset = offset >>> 0
  if (!noAssert) checkOffset(offset, 8, this.length)
  return ieee754.read(this, offset, false, 52, 8)
}

function checkInt (buf, value, offset, ext, max, min) {
  if (!Buffer.isBuffer(buf)) throw new TypeError('"buffer" argument must be a Buffer instance')
  if (value > max || value < min) throw new RangeError('"value" argument is out of bounds')
  if (offset + ext > buf.length) throw new RangeError('Index out of range')
}

Buffer.prototype.writeUintLE =
Buffer.prototype.writeUIntLE = function writeUIntLE (value, offset, byteLength, noAssert) {
  value = +value
  offset = offset >>> 0
  byteLength = byteLength >>> 0
  if (!noAssert) {
    var maxBytes = Math.pow(2, 8 * byteLength) - 1
    checkInt(this, value, offset, byteLength, maxBytes, 0)
  }

  var mul = 1
  var i = 0
  this[offset] = value & 0xFF
  while (++i < byteLength && (mul *= 0x100)) {
    this[offset + i] = (value / mul) & 0xFF
  }

  return offset + byteLength
}

Buffer.prototype.writeUintBE =
Buffer.prototype.writeUIntBE = function writeUIntBE (value, offset, byteLength, noAssert) {
  value = +value
  offset = offset >>> 0
  byteLength = byteLength >>> 0
  if (!noAssert) {
    var maxBytes = Math.pow(2, 8 * byteLength) - 1
    checkInt(this, value, offset, byteLength, maxBytes, 0)
  }

  var i = byteLength - 1
  var mul = 1
  this[offset + i] = value & 0xFF
  while (--i >= 0 && (mul *= 0x100)) {
    this[offset + i] = (value / mul) & 0xFF
  }

  return offset + byteLength
}

Buffer.prototype.writeUint8 =
Buffer.prototype.writeUInt8 = function writeUInt8 (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 1, 0xff, 0)
  this[offset] = (value & 0xff)
  return offset + 1
}

Buffer.prototype.writeUint16LE =
Buffer.prototype.writeUInt16LE = function writeUInt16LE (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 2, 0xffff, 0)
  this[offset] = (value & 0xff)
  this[offset + 1] = (value >>> 8)
  return offset + 2
}

Buffer.prototype.writeUint16BE =
Buffer.prototype.writeUInt16BE = function writeUInt16BE (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 2, 0xffff, 0)
  this[offset] = (value >>> 8)
  this[offset + 1] = (value & 0xff)
  return offset + 2
}

Buffer.prototype.writeUint32LE =
Buffer.prototype.writeUInt32LE = function writeUInt32LE (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 4, 0xffffffff, 0)
  this[offset + 3] = (value >>> 24)
  this[offset + 2] = (value >>> 16)
  this[offset + 1] = (value >>> 8)
  this[offset] = (value & 0xff)
  return offset + 4
}

Buffer.prototype.writeUint32BE =
Buffer.prototype.writeUInt32BE = function writeUInt32BE (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 4, 0xffffffff, 0)
  this[offset] = (value >>> 24)
  this[offset + 1] = (value >>> 16)
  this[offset + 2] = (value >>> 8)
  this[offset + 3] = (value & 0xff)
  return offset + 4
}

Buffer.prototype.writeIntLE = function writeIntLE (value, offset, byteLength, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) {
    var limit = Math.pow(2, (8 * byteLength) - 1)

    checkInt(this, value, offset, byteLength, limit - 1, -limit)
  }

  var i = 0
  var mul = 1
  var sub = 0
  this[offset] = value & 0xFF
  while (++i < byteLength && (mul *= 0x100)) {
    if (value < 0 && sub === 0 && this[offset + i - 1] !== 0) {
      sub = 1
    }
    this[offset + i] = ((value / mul) >> 0) - sub & 0xFF
  }

  return offset + byteLength
}

Buffer.prototype.writeIntBE = function writeIntBE (value, offset, byteLength, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) {
    var limit = Math.pow(2, (8 * byteLength) - 1)

    checkInt(this, value, offset, byteLength, limit - 1, -limit)
  }

  var i = byteLength - 1
  var mul = 1
  var sub = 0
  this[offset + i] = value & 0xFF
  while (--i >= 0 && (mul *= 0x100)) {
    if (value < 0 && sub === 0 && this[offset + i + 1] !== 0) {
      sub = 1
    }
    this[offset + i] = ((value / mul) >> 0) - sub & 0xFF
  }

  return offset + byteLength
}

Buffer.prototype.writeInt8 = function writeInt8 (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 1, 0x7f, -0x80)
  if (value < 0) value = 0xff + value + 1
  this[offset] = (value & 0xff)
  return offset + 1
}

Buffer.prototype.writeInt16LE = function writeInt16LE (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 2, 0x7fff, -0x8000)
  this[offset] = (value & 0xff)
  this[offset + 1] = (value >>> 8)
  return offset + 2
}

Buffer.prototype.writeInt16BE = function writeInt16BE (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 2, 0x7fff, -0x8000)
  this[offset] = (value >>> 8)
  this[offset + 1] = (value & 0xff)
  return offset + 2
}

Buffer.prototype.writeInt32LE = function writeInt32LE (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 4, 0x7fffffff, -0x80000000)
  this[offset] = (value & 0xff)
  this[offset + 1] = (value >>> 8)
  this[offset + 2] = (value >>> 16)
  this[offset + 3] = (value >>> 24)
  return offset + 4
}

Buffer.prototype.writeInt32BE = function writeInt32BE (value, offset, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) checkInt(this, value, offset, 4, 0x7fffffff, -0x80000000)
  if (value < 0) value = 0xffffffff + value + 1
  this[offset] = (value >>> 24)
  this[offset + 1] = (value >>> 16)
  this[offset + 2] = (value >>> 8)
  this[offset + 3] = (value & 0xff)
  return offset + 4
}

function checkIEEE754 (buf, value, offset, ext, max, min) {
  if (offset + ext > buf.length) throw new RangeError('Index out of range')
  if (offset < 0) throw new RangeError('Index out of range')
}

function writeFloat (buf, value, offset, littleEndian, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) {
    checkIEEE754(buf, value, offset, 4, 3.4028234663852886e+38, -3.4028234663852886e+38)
  }
  ieee754.write(buf, value, offset, littleEndian, 23, 4)
  return offset + 4
}

Buffer.prototype.writeFloatLE = function writeFloatLE (value, offset, noAssert) {
  return writeFloat(this, value, offset, true, noAssert)
}

Buffer.prototype.writeFloatBE = function writeFloatBE (value, offset, noAssert) {
  return writeFloat(this, value, offset, false, noAssert)
}

function writeDouble (buf, value, offset, littleEndian, noAssert) {
  value = +value
  offset = offset >>> 0
  if (!noAssert) {
    checkIEEE754(buf, value, offset, 8, 1.7976931348623157E+308, -1.7976931348623157E+308)
  }
  ieee754.write(buf, value, offset, littleEndian, 52, 8)
  return offset + 8
}

Buffer.prototype.writeDoubleLE = function writeDoubleLE (value, offset, noAssert) {
  return writeDouble(this, value, offset, true, noAssert)
}

Buffer.prototype.writeDoubleBE = function writeDoubleBE (value, offset, noAssert) {
  return writeDouble(this, value, offset, false, noAssert)
}

// copy(targetBuffer, targetStart=0, sourceStart=0, sourceEnd=buffer.length)
Buffer.prototype.copy = function copy (target, targetStart, start, end) {
  if (!Buffer.isBuffer(target)) throw new TypeError('argument should be a Buffer')
  if (!start) start = 0
  if (!end && end !== 0) end = this.length
  if (targetStart >= target.length) targetStart = target.length
  if (!targetStart) targetStart = 0
  if (end > 0 && end < start) end = start

  // Copy 0 bytes; we're done
  if (end === start) return 0
  if (target.length === 0 || this.length === 0) return 0

  // Fatal error conditions
  if (targetStart < 0) {
    throw new RangeError('targetStart out of bounds')
  }
  if (start < 0 || start >= this.length) throw new RangeError('Index out of range')
  if (end < 0) throw new RangeError('sourceEnd out of bounds')

  // Are we oob?
  if (end > this.length) end = this.length
  if (target.length - targetStart < end - start) {
    end = target.length - targetStart + start
  }

  var len = end - start

  if (this === target && typeof Uint8Array.prototype.copyWithin === 'function') {
    // Use built-in when available, missing from IE11
    this.copyWithin(targetStart, start, end)
  } else {
    Uint8Array.prototype.set.call(
      target,
      this.subarray(start, end),
      targetStart
    )
  }

  return len
}

// Usage:
//    buffer.fill(number[, offset[, end]])
//    buffer.fill(buffer[, offset[, end]])
//    buffer.fill(string[, offset[, end]][, encoding])
Buffer.prototype.fill = function fill (val, start, end, encoding) {
  // Handle string cases:
  if (typeof val === 'string') {
    if (typeof start === 'string') {
      encoding = start
      start = 0
      end = this.length
    } else if (typeof end === 'string') {
      encoding = end
      end = this.length
    }
    if (encoding !== undefined && typeof encoding !== 'string') {
      throw new TypeError('encoding must be a string')
    }
    if (typeof encoding === 'string' && !Buffer.isEncoding(encoding)) {
      throw new TypeError('Unknown encoding: ' + encoding)
    }
    if (val.length === 1) {
      var code = val.charCodeAt(0)
      if ((encoding === 'utf8' && code < 128) ||
          encoding === 'latin1') {
        // Fast path: If `val` fits into a single byte, use that numeric value.
        val = code
      }
    }
  } else if (typeof val === 'number') {
    val = val & 255
  } else if (typeof val === 'boolean') {
    val = Number(val)
  }

  // Invalid ranges are not set to a default, so can range check early.
  if (start < 0 || this.length < start || this.length < end) {
    throw new RangeError('Out of range index')
  }

  if (end <= start) {
    return this
  }

  start = start >>> 0
  end = end === undefined ? this.length : end >>> 0

  if (!val) val = 0

  var i
  if (typeof val === 'number') {
    for (i = start; i < end; ++i) {
      this[i] = val
    }
  } else {
    var bytes = Buffer.isBuffer(val)
      ? val
      : Buffer.from(val, encoding)
    var len = bytes.length
    if (len === 0) {
      throw new TypeError('The value "' + val +
        '" is invalid for argument "value"')
    }
    for (i = 0; i < end - start; ++i) {
      this[i + start] = bytes[i % len]
    }
  }

  return this
}

// HELPER FUNCTIONS
// ================

var INVALID_BASE64_RE = /[^+/0-9A-Za-z-_]/g

function base64clean (str) {
  // Node takes equal signs as end of the Base64 encoding
  str = str.split('=')[0]
  // Node strips out invalid characters like \n and \t from the string, base64-js does not
  str = str.trim().replace(INVALID_BASE64_RE, '')
  // Node converts strings with length < 2 to ''
  if (str.length < 2) return ''
  // Node allows for non-padded base64 strings (missing trailing ===), base64-js does not
  while (str.length % 4 !== 0) {
    str = str + '='
  }
  return str
}

function utf8ToBytes (string, units) {
  units = units || Infinity
  var codePoint
  var length = string.length
  var leadSurrogate = null
  var bytes = []

  for (var i = 0; i < length; ++i) {
    codePoint = string.charCodeAt(i)

    // is surrogate component
    if (codePoint > 0xD7FF && codePoint < 0xE000) {
      // last char was a lead
      if (!leadSurrogate) {
        // no lead yet
        if (codePoint > 0xDBFF) {
          // unexpected trail
          if ((units -= 3) > -1) bytes.push(0xEF, 0xBF, 0xBD)
          continue
        } else if (i + 1 === length) {
          // unpaired lead
          if ((units -= 3) > -1) bytes.push(0xEF, 0xBF, 0xBD)
          continue
        }

        // valid lead
        leadSurrogate = codePoint

        continue
      }

      // 2 leads in a row
      if (codePoint < 0xDC00) {
        if ((units -= 3) > -1) bytes.push(0xEF, 0xBF, 0xBD)
        leadSurrogate = codePoint
        continue
      }

      // valid surrogate pair
      codePoint = (leadSurrogate - 0xD800 << 10 | codePoint - 0xDC00) + 0x10000
    } else if (leadSurrogate) {
      // valid bmp char, but last char was a lead
      if ((units -= 3) > -1) bytes.push(0xEF, 0xBF, 0xBD)
    }

    leadSurrogate = null

    // encode utf8
    if (codePoint < 0x80) {
      if ((units -= 1) < 0) break
      bytes.push(codePoint)
    } else if (codePoint < 0x800) {
      if ((units -= 2) < 0) break
      bytes.push(
        codePoint >> 0x6 | 0xC0,
        codePoint & 0x3F | 0x80
      )
    } else if (codePoint < 0x10000) {
      if ((units -= 3) < 0) break
      bytes.push(
        codePoint >> 0xC | 0xE0,
        codePoint >> 0x6 & 0x3F | 0x80,
        codePoint & 0x3F | 0x80
      )
    } else if (codePoint < 0x110000) {
      if ((units -= 4) < 0) break
      bytes.push(
        codePoint >> 0x12 | 0xF0,
        codePoint >> 0xC & 0x3F | 0x80,
        codePoint >> 0x6 & 0x3F | 0x80,
        codePoint & 0x3F | 0x80
      )
    } else {
      throw new Error('Invalid code point')
    }
  }

  return bytes
}

function asciiToBytes (str) {
  var byteArray = []
  for (var i = 0; i < str.length; ++i) {
    // Node's code seems to be doing this and not & 0x7F..
    byteArray.push(str.charCodeAt(i) & 0xFF)
  }
  return byteArray
}

function utf16leToBytes (str, units) {
  var c, hi, lo
  var byteArray = []
  for (var i = 0; i < str.length; ++i) {
    if ((units -= 2) < 0) break

    c = str.charCodeAt(i)
    hi = c >> 8
    lo = c % 256
    byteArray.push(lo)
    byteArray.push(hi)
  }

  return byteArray
}

function base64ToBytes (str) {
  return base64.toByteArray(base64clean(str))
}

function blitBuffer (src, dst, offset, length) {
  for (var i = 0; i < length; ++i) {
    if ((i + offset >= dst.length) || (i >= src.length)) break
    dst[i + offset] = src[i]
  }
  return i
}

// ArrayBuffer or Uint8Array objects from other contexts (i.e. iframes) do not pass
// the `instanceof` check but they should be treated as of that type.
// See: https://github.com/feross/buffer/issues/166
function isInstance (obj, type) {
  return obj instanceof type ||
    (obj != null && obj.constructor != null && obj.constructor.name != null &&
      obj.constructor.name === type.name)
}
function numberIsNaN (obj) {
  // For IE11 support
  return obj !== obj // eslint-disable-line no-self-compare
}

// Create lookup table for `toString('hex')`
// See: https://github.com/feross/buffer/issues/219
var hexSliceLookupTable = (function () {
  var alphabet = '0123456789abcdef'
  var table = new Array(256)
  for (var i = 0; i < 16; ++i) {
    var i16 = i * 16
    for (var j = 0; j < 16; ++j) {
      table[i16 + j] = alphabet[i] + alphabet[j]
    }
  }
  return table
})()

}).call(this)}).call(this,require("buffer").Buffer)

},{"base64-js":4,"buffer":5,"ieee754":7}],7:[function(require,module,exports){
/*! ieee754. BSD-3-Clause License. Feross Aboukhadijeh <https://feross.org/opensource> */
exports.read = function (buffer, offset, isLE, mLen, nBytes) {
  var e, m
  var eLen = (nBytes * 8) - mLen - 1
  var eMax = (1 << eLen) - 1
  var eBias = eMax >> 1
  var nBits = -7
  var i = isLE ? (nBytes - 1) : 0
  var d = isLE ? -1 : 1
  var s = buffer[offset + i]

  i += d

  e = s & ((1 << (-nBits)) - 1)
  s >>= (-nBits)
  nBits += eLen
  for (; nBits > 0; e = (e * 256) + buffer[offset + i], i += d, nBits -= 8) {}

  m = e & ((1 << (-nBits)) - 1)
  e >>= (-nBits)
  nBits += mLen
  for (; nBits > 0; m = (m * 256) + buffer[offset + i], i += d, nBits -= 8) {}

  if (e === 0) {
    e = 1 - eBias
  } else if (e === eMax) {
    return m ? NaN : ((s ? -1 : 1) * Infinity)
  } else {
    m = m + Math.pow(2, mLen)
    e = e - eBias
  }
  return (s ? -1 : 1) * m * Math.pow(2, e - mLen)
}

exports.write = function (buffer, value, offset, isLE, mLen, nBytes) {
  var e, m, c
  var eLen = (nBytes * 8) - mLen - 1
  var eMax = (1 << eLen) - 1
  var eBias = eMax >> 1
  var rt = (mLen === 23 ? Math.pow(2, -24) - Math.pow(2, -77) : 0)
  var i = isLE ? 0 : (nBytes - 1)
  var d = isLE ? 1 : -1
  var s = value < 0 || (value === 0 && 1 / value < 0) ? 1 : 0

  value = Math.abs(value)

  if (isNaN(value) || value === Infinity) {
    m = isNaN(value) ? 1 : 0
    e = eMax
  } else {
    e = Math.floor(Math.log(value) / Math.LN2)
    if (value * (c = Math.pow(2, -e)) < 1) {
      e--
      c *= 2
    }
    if (e + eBias >= 1) {
      value += rt / c
    } else {
      value += rt * Math.pow(2, 1 - eBias)
    }
    if (value * c >= 2) {
      e++
      c /= 2
    }

    if (e + eBias >= eMax) {
      m = 0
      e = eMax
    } else if (e + eBias >= 1) {
      m = ((value * c) - 1) * Math.pow(2, mLen)
      e = e + eBias
    } else {
      m = value * Math.pow(2, eBias - 1) * Math.pow(2, mLen)
      e = 0
    }
  }

  for (; mLen >= 8; buffer[offset + i] = m & 0xff, i += d, m /= 256, mLen -= 8) {}

  e = (e << mLen) | m
  eLen += mLen
  for (; eLen > 0; buffer[offset + i] = e & 0xff, i += d, e /= 256, eLen -= 8) {}

  buffer[offset + i - d] |= s * 128
}

},{}],8:[function(require,module,exports){
"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.hook_bi = hook_bi;
exports.hook_binder_transact = hook_binder_transact;
exports.hookHandleBindApplication = hookHandleBindApplication;
exports.register_request_verification_code = register_request_verification_code;
exports.register_verification_code = register_verification_code;
const common = __importStar(require("../../my_modules/common"));
/********************************************
 *
 * hook 2.25.31.72 版本注册的参数
 *
 *******************************************/
var fake_bi = -1;
function hook_bi() {
    // libwhatsapp.so 2.25.31.72 0x4AAFE4
    var func = Module.findExportByName("libc.so", "fopen");
    var fakeBootId = "123e4567-e89b-12d3-a456-426614174000";
    if (func) {
        Interceptor.attach(func, {
            onEnter: function (args) {
                var path = args[0].readUtf8String();
                if (path === "/proc/sys/kernel/random/boot_id") {
                    // Create a fake file descriptor
                    this.fakeFd = true;
                    console.log("[boot_id] fopen boot id");
                }
            },
            onLeave: function (retval) {
                if (this.fakeFd) {
                    // Return a fake file descriptor (e.g., 100)
                    fake_bi = Number(retval);
                    console.log("[boot_id] boot id fd: ", fake_bi);
                }
            },
        });
    }
    else {
        console.log("fopen Function not found");
    }
    // Hook fread function to return fake boot_id content
    // size_t fread(void *ptr, size_t size, size_t n, FILE *stream)
    var freadFunc = Module.findExportByName("libc.so", "fread");
    if (freadFunc) {
        Interceptor.attach(freadFunc, {
            onEnter: function (args) {
                this.buffer = args[0];
                this.size = args[1].toInt32();
                this.n = args[2].toInt32();
                this.fd = args[3];
            },
            onLeave: function (retval) {
                if (Number(this.fd) == fake_bi) {
                    // console.log(`[boot_id]\tsize: ${this.size} n: ${this.n}\n${this.buffer.readCString()}`);
                    var content = common.uuid() + "\n";
                    console.log("[boot_id] fake boot id: " + content);
                    var contentLength = content.length;
                    var bytesToCopy = Math.min(contentLength, this.size * this.n);
                    this.buffer.writeUtf8String(content);
                    console.log(`[boot_id]\n${hexdump(this.buffer, { length: Number(retval) })}`);
                    var elementsRead = Math.floor(bytesToCopy / this.size);
                    retval.replace(ptr(elementsRead));
                    fake_bi = -1;
                }
            },
        });
    }
    else {
        console.log("fread Function not found");
    }
}
var fakeSourceDir = "/data/app/fake_path/base.apk";
var fakeSourceDirTimes = 0;
function hook_binder_transact() {
    Java.perform(function () {
        // Define fakeSignature - replace with your actual fake signature hex string
        var fakeSignatureHex = "beafbeaf"; // e.g., a valid hex representation of signature bytes
        var SignatureClass = Java.use("android.content.pm.Signature");
        var fakeSignature = Java.array("android.content.pm.Signature", [SignatureClass.$new(fakeSignatureHex)]);
        // Get current package name
        var ActivityThread = Java.use("android.app.ActivityThread");
        var app = ActivityThread.currentApplication();
        // var context = app.getApplicationContext();
        // var currentPackage = context.getPackageName();
        var currentPackage = "com.whatsapp";
        // Classes
        var BinderProxy = Java.use("android.os.BinderProxy");
        var Parcel = Java.use("android.os.Parcel");
        var PackageInfo = Java.use("android.content.pm.PackageInfo");
        var Build = Java.use("android.os.Build");
        var Build_VERSION = Java.use("android.os.Build$VERSION");
        var ApplicationInfo = Java.use("android.content.pm.ApplicationInfo");
        // Constants
        var INTERFACE_TRANSACTION = 1598968902; // IBinder.INTERFACE_TRANSACTION
        var TRANSACTION_getPackageInfo = 3; // Android 14 Adjust based on Android version; 3 in recent versions, may be 1 or 2 in older
        var TRANSACTION_getApplicationInfo = 9; // Android 14
        // Hook transact
        BinderProxy.transact.implementation = function (code, data, reply, flags) {
            var result = this.transact(code, data, reply, flags);
            if (data === null || reply === null) {
                return result;
            }
            if (code === INTERFACE_TRANSACTION) {
                return result;
            }
            var desc = this.getInterfaceDescriptor();
            if (desc === null || desc !== "android.content.pm.IPackageManager") {
                return result;
            }
            if (code === TRANSACTION_getPackageInfo) {
                reply.readException();
                if (reply.readInt() !== 0) {
                    var packageInfo = PackageInfo.CREATOR.value.createFromParcel(reply);
                    packageInfo = Java.cast(packageInfo, PackageInfo);
                    console.log("packageInfo: " + JSON.stringify(packageInfo) + "\tpackageName: " + packageInfo.packageName.value);
                    if (String(packageInfo.packageName.value) == currentPackage) {
                        if (fakeSignature[0] === null) {
                            console.error(">>>>>>>>>> byPassSignature fakeSignature == null");
                            Java.use("android.os.Process").killProcess(Java.use("android.os.Process").myPid());
                            return result;
                        }
                        if (packageInfo.signatures.value !== null && packageInfo.signatures.value.length > 0) {
                            // packageInfo.signatures.value[0] = fakeSignature[0];
                            console.log(">>>>>>>>>> byPassSignature fakeSignature[0] = " + fakeSignature[0]);
                        }
                        if (Build_VERSION.SDK_INT >= 28) {
                            // Build.VERSION_CODES.P
                            if (packageInfo.signingInfo.value !== null) {
                                var SigningInfo = Java.use("android.content.pm.SigningInfo");
                                var signaturesArray = packageInfo.signingInfo.value.getApkContentsSigners();
                                if (signaturesArray !== null && signaturesArray.length > 0) {
                                    // signaturesArray[0] = fakeSignature[0];
                                    console.log(">>>>>>>>>> byPassSignature fakeSignature[0] = " + fakeSignature[0]);
                                }
                            }
                        }
                        reply.setDataPosition(0);
                        reply.setDataSize(0);
                        reply.writeNoException();
                        reply.writeInt(1);
                        packageInfo.writeToParcel(reply, 1); // PARCELABLE_WRITE_RETURN_VALUE = 1
                    }
                }
                reply.setDataPosition(0);
            }
            else if (code === TRANSACTION_getApplicationInfo) {
                reply.readException();
                if (reply.readInt() !== 0) {
                    // 如果有数据
                    // console.log("ApplicationInfo.CREATOR: " + JSON.stringify(ApplicationInfo.CREATOR.value))
                    var appInfo = ApplicationInfo.CREATOR.value.createFromParcel(reply);
                    // console.log("appInfo: " + JSON.stringify(appInfo))
                    appInfo = Java.cast(appInfo, ApplicationInfo);
                    console.log("appInfo: " + JSON.stringify(appInfo) + "\tpackageName: " + appInfo.packageName.value);
                    if (String(appInfo.packageName.value) == currentPackage) {
                        // 修改 sourceDir
                        var log_str = "[getApplicationInfo] Real sourceDir: " + appInfo.sourceDir.value + "\n";
                        appInfo.sourceDir.value = fakeSourceDir;
                        log_str += "[getApplicationInfo] Modified sourceDir to: " + fakeSourceDir + "\n";
                        log_str += "[getApplicationInfo] Call Stack: " + common.getJavaStackTraceString();
                        console.log(log_str);
                        // 可选：修改 publicSourceDir
                        appInfo.publicSourceDir.value = fakeSourceDir;
                        // 重写 Parcel
                        reply.setDataPosition(0);
                        reply.setDataSize(0);
                        reply.writeNoException();
                        reply.writeInt(1);
                        appInfo.writeToParcel(reply, 1); // PARCELABLE_WRITE_RETURN_VALUE = 1
                        fakeSourceDir += "." + String(fakeSourceDirTimes);
                        fakeSourceDirTimes += 1;
                    }
                }
                reply.setDataPosition(0);
            }
            return result;
        };
    });
}
// hook sourceDir 会导致 "_p"，"p"，"sizeInBytes", "shatr" 等值为空。暂时不用管，错误值也能下码
// 看 libwhatsapp 2.25.31.72 0x46A370
function hookHandleBindApplication() {
    // Java.perform(function () {
    var fakeSourceDir = "/data/app/fake_path/base.apk"; // Your fake path
    var ActivityThread = Java.use("android.app.ActivityThread");
    // Hook handleBindApplication (takes AppBindData as arg)
    ActivityThread.handleBindApplication.implementation = function (data) {
        // Call original method first
        var result = this.handleBindApplication(data);
        //     // Now mBoundApplication is set; access it safely
        var boundApp = this.mBoundApplication.value;
        if (boundApp !== null) {
            var loadedApk = boundApp.info.value;
            if (loadedApk !== null) {
                var appInfo = loadedApk.mApplicationInfo.value;
                if (appInfo !== null) {
                    var originalSourceDir = appInfo.sourceDir.value;
                    appInfo.sourceDir.value = fakeSourceDir;
                    appInfo.publicSourceDir.value = fakeSourceDir; // Optional
                    console.log("[hookBindApp] Original sourceDir: " + originalSourceDir + " -> Modified to: " + fakeSourceDir);
                    console.log("[hookBindApp] Call Stack: " + common.getJavaStackTraceString());
                }
                else {
                    console.error("[hookBindApp] appInfo is null");
                }
            }
            else {
                console.error("[hookBindApp] loadedApk is null");
            }
        }
        else {
            console.error("[hookBindApp] boundApp is null");
        }
        return result;
    };
    // });
}
function hook_cert() {
    var C15u = Java.use("X.15u");
    C15u["A07"].implementation = function (bArr, bArr2) {
        console.log(`C15u.A07 is called: bArr=\n${common.byteToHexString(bArr)}, bArr2=\n${common.byteToHexString(bArr2)}`);
        let result = this["A07"](bArr, bArr2);
        console.log(`C15u.A07 result=\n${common.byteToHexString(result)}`);
        return result;
    };
    var C15u = Java.use("X.15u");
    C15u["A03"].implementation = function (c15u, num, bArr) {
        console.log(`C15u.A03 is called: c15u=${c15u}, num=${num}, bArr=${bArr}`);
        let result = this["A03"](c15u, num, bArr);
        console.log(`C15u.A03 result=${common.byteToHexString(result)}`);
        return result;
    };
}
function start_register_phone() {
    Java.perform(function () {
        hook_verify_phone_number("123456");
        hook_confirm_dialog_v2();
        registration("1", "5812802680");
        skip_eula();
    });
}
function register_request_verification_code(country_code, phone_number) {
    Java.perform(function () {
        hook_confirm_dialog_v2();
        registration(country_code, phone_number);
        skip_eula();
    });
}
function register_verification_code(code) {
    Java.perform(function () {
        hook_verify_phone_number("123456");
    });
}
function skip_eula() {
    var hasClicked = false;
    console.log("[+] Frida 脚本已加载。等待 EULA Activity 获取焦点...");
    var EULA = Java.use("com.whatsapp.registration.app.EULA");
    // Hook onWindowFocusChanged，这是一个更安全的 UI 操作时机
    EULA.onWindowFocusChanged.overload("boolean").implementation = function (hasFocus) {
        // 首先，必须调用原始方法
        this.onWindowFocusChanged(hasFocus);
        // 如果 Activity 没有获得焦点，或者我们已经点击过了，就直接返回
        if (!hasFocus || hasClicked) {
            return;
        }
        console.log("[+] EULA.onWindowFocusChanged(true) 已 Hook。准备模拟点击...");
        // 设置标志位，防止重复执行
        hasClicked = true;
        var eulaActivity = this; // 'this' 是 EULA Activity 的实例
        // 这个方法本身就在 UI 线程上，所以不需要再调度
        try {
            var packageName = eulaActivity.getPackageName();
            var eulaAcceptButtonId = eulaActivity.getResources().getIdentifier("eula_accept", "id", packageName);
            if (eulaAcceptButtonId !== 0) {
                console.log("[+] 找到 'eula_accept' 按钮的 ID: " + eulaAcceptButtonId);
                var agreeButton = eulaActivity.findViewById(eulaAcceptButtonId);
                if (agreeButton) {
                    console.log("[+] '同意并继续' 按钮已找到。正在执行 .performClick() ...");
                    agreeButton.performClick();
                    console.log("[+] 成功调用 .performClick()。");
                }
                else {
                    console.log("[-] 错误：未能通过 findViewById 找到 '同意并继续' 按钮。");
                }
            }
            else {
                console.log("[-] 错误：未能找到 'eula_accept' 的资源 ID。");
            }
        }
        catch (e) {
            console.log("[-] UI 交互过程中发生错误: " + e.message);
        }
    };
}
function registration(country_code, phone_number) {
    var phoneEntered = false;
    var RegisterPhone = Java.use("com.whatsapp.registration.app.phonenumberentry.RegisterPhone");
    var AbstractActivityC29204Ebj = Java.use("X.Ebj");
    var String = Java.use("java.lang.String");
    RegisterPhone.onResume.implementation = function () {
        // console.log("[+] this " + JSON.stringify(this));
        this.onResume();
        if (!RegisterPhone.class.isInstance(this)) {
            console.log("[!] Hook 在一个意外的对象上触发了: " + this.getClass().getName() + "，已跳过。");
            return;
        }
        if (phoneEntered) {
            console.log("[!] Hook 已触发，跳过。");
            return;
        }
        phoneEntered = true;
        var activity = this;
        // Java.scheduleOnMainThread(function () {
        // 1. 从 activity.A0G 获取 F87 实例
        // console.log("[+] casting... " + JSON.stringify(activity));
        var activityAsSuper = Java.cast(activity, AbstractActivityC29204Ebj);
        // console.log("[+] cast " + JSON.stringify(activityAsSuper));
        var f87_instance = activityAsSuper.A0G.value;
        if (!f87_instance) {
            console.log("[-] 错误: 无法从父类的 A0G 字段获取 F87 实例。");
            return;
        }
        // 2. 从 F87 实例的 A05 字段获取 PhoneNumberEntry 视图
        var phoneNumberEntryView = f87_instance.A05.value;
        if (!phoneNumberEntryView) {
            console.log("[-] 错误: 无法从 f87_instance.A05 获取 PhoneNumberEntry 视图。");
            return;
        }
        console.log("[+] 成功定位到 PhoneNumberEntry 视图。");
        var packageName = activity.getPackageName();
        // 3. 在 PhoneNumberEntry 视图内部查找国家代码和手机号的 EditText
        var cc_id = activity.getResources().getIdentifier("registration_cc", "id", packageName);
        var phone_id = activity.getResources().getIdentifier("registration_phone", "id", packageName);
        // 使用 phoneNumberEntryView 而不是 activity 来查找
        var countryCodeEditText = phoneNumberEntryView.findViewById(cc_id);
        var phoneNumberEditText = phoneNumberEntryView.findViewById(phone_id);
        if (!countryCodeEditText || !phoneNumberEditText) {
            console.log("[-] 错误: 在 PhoneNumberEntry 内部未能找到 registration_cc 或 registration_phone。");
            return;
        }
        console.log("[+] 成功定位到国家代码和手机号码的 EditText。");
        // 4. 填写号码
        // console.log("[+] WaEditText: " + JSON.stringify(countryCodeEditText));
        countryCodeEditText = Java.cast(countryCodeEditText, Java.use("android.widget.TextView"));
        phoneNumberEditText = Java.cast(phoneNumberEditText, Java.use("android.widget.TextView"));
        countryCodeEditText.setText.overload("java.lang.CharSequence").call(countryCodeEditText, String.$new(country_code));
        phoneNumberEditText.setText.overload("java.lang.CharSequence").call(phoneNumberEditText, String.$new(phone_number));
        console.log("[+] 已成功设置国家代码和手机号码。");
        // 5. 找到并点击提交按钮 (这个按钮在 Activity 的主布局中)
        var submitButtonId = activity.getResources().getIdentifier("registration_submit", "id", packageName);
        if (submitButtonId) {
            var submitButton = activity.findViewById(submitButtonId);
            if (submitButton) {
                submitButton.performClick();
                console.log("[+] 成功点击 '下一步' 按钮。注册请求已发起！");
            }
            else {
                console.log("[-] 错误: 未找到提交按钮的 View。");
            }
        }
        else {
            console.log("[-] 错误: 未找到 'registration_submit' 按钮的 ID。");
        }
        // });
    };
}
/**
RegisterPhone.java
```java
@Override // X.AbstractActivityC29204Ebj, android.app.Activity
public Dialog onCreateDialog(int i) throws Resources.NotFoundException {
    C28019DoV c28019DoVA00;
    DialogInterfaceC28023DoZ dialogInterfaceC28023DoZCreate;

    // ... 其他 case ...
    // 当传入的 ID 是 0x15 (十进制的 21) 时：
    if (i == 0x15) {
         Log.i("RegisterPhone/dialog/num_confirm"); // 日志明确说明了这是“号码确认”对话框

         // ... 省略了创建对话框UI的复杂代码 ...
         // 设置“确定”按钮 (OK button)
         // 它的点击事件会触发一个后续操作 (我们分析过是 A10() 方法)
         c28019DoVA00.A0Z(new DialogInterfaceOnClickListenerC31192FZl(this, 0x26), R.string._name_removed__res_0x7f122b05);
         dialogInterfaceC28023DoZCreate = c28019DoVA00.create();
         this.A09 = dialogInterfaceC28023DoZCreate; // 将创建的对话框存入 A09 字段
         return dialogInterfaceC28023DoZCreate;
     }
     // ... 其他 case ...
     return super.onCreateDialog(i);
 }
 ````
 */
function hook_confirm_dialog_v2() {
    var RegisterPhone = Java.use("com.whatsapp.registration.app.phonenumberentry.RegisterPhone");
    var Activity = Java.use("android.app.Activity");
    Activity.showDialog.overload("int").implementation = function (id) {
        // 检查是否是在 RegisterPhone 中调用，并且对话框 ID 是否为 21 (0x15)
        if (RegisterPhone.class.isInstance(this) && id === 21) {
            console.log("[3/3] 拦截到号码确认对话框 (ID: 21)，正在跳过...");
            // 直接调用“确定”按钮的后续方法 A10()
            RegisterPhone.A10(this);
            console.log("[+] 已手动调用 A10()，注册流程将继续。");
            // 通过不调用 this.showDialog(id)，我们阻止了对话框的显示
            return;
        }
        // 对于其他所有对话框，正常显示
        return this.showDialog(id);
    };
}
function hook_confirm_dialog() {
    // --- 步骤 3: Hook Dialog 对话框，自动点击 "确定" ---
    var Dialog = Java.use("android.app.Dialog");
    var DialogInterface = Java.use("android.content.DialogInterface");
    // var dialogClicked = false;
    Dialog.show.implementation = function () {
        // 先调用原始的 show 方法
        this.show();
        // if (dialogClicked) return;
        var dialog = this;
        console.log("[3/4] 检测到对话框显示，准备点击'确定'...");
        // Java.scheduleOnMainThread(function () {
        // BUTTON_POSITIVE 通常代表“确定”、“OK”等积极操作的按钮
        var positiveButtonId = Java.use("android.R$id").button1.value;
        var positiveButton = dialog.findViewById(positiveButtonId);
        positiveButton = Java.cast(positiveButton, Java.use("android.widget.Button"));
        // console.log("positiveButton: " + JSON.stringify(positiveButton));
        if (positiveButton) {
            // dialogClicked = true; // 确保只点击一次
            console.log("[+] 找到对话框的 '确定' 按钮, 正在点击...");
            positiveButton.performClick();
            console.log("[+] 成功点击'确定'。最终注册请求已发出！");
        }
        // });
    };
}
function hook_verify_phone_number(verificationCode) {
    var verificationCodeEntered = false;
    // --- 步骤 3: Hook VerifyPhoneNumber (新) ---
    var StringJava = Java.use("java.lang.String");
    var VerifyPhoneNumber = Java.use("com.whatsapp.registration.app.verifyphone.VerifyPhoneNumber");
    var CodeInputField = Java.use("com.whatsapp.CodeInputField");
    var CharSequence = Java.use("java.lang.CharSequence"); // CodeInputField extends EditText, which implements CharSequence
    // CodeInputField 继承自 EditText，而 EditText 继承自 TextView，TextView 有多个 setText 方法
    // 猜测 CodeInputField 也会有 setText(CharSequence) 方法。
    // 如果再次出现 TypeError: not a function，则切换到 setText.overload('java.lang.CharSequence', 'android.widget.TextView$BufferType')
    VerifyPhoneNumber.onResume.implementation = function () {
        this.onResume(); // 调用原始的 onResume
        if (verificationCodeEntered)
            return;
        // 类型检查
        if (!VerifyPhoneNumber.class.isInstance(this)) {
            console.log("[!] VerifyPhoneNumber hook 意外触发在: " + this.getClass().getName() + "。跳过。");
            return;
        }
        verificationCodeEntered = true;
        console.log("[4/4] VerifyPhoneNumber: onResume. 正在输入验证码...");
        var activity = this;
        var codeInputField = activity.A0J.value; // 获取 CodeInputField 实例
        if (!codeInputField) {
            console.log("[-] 错误: CodeInputField (A0J) 未找到。");
            return;
        }
        // 假设验证码为 6 位数字
        // var verificationCode = "123456";
        // 尝试调用 setText(CharSequence) 方法
        codeInputField = Java.cast(codeInputField, Java.use("android.widget.TextView"));
        codeInputField.setText.overload("java.lang.CharSequence").call(codeInputField, StringJava.$new(verificationCode));
        // 如果这里报错 'TypeError: not a function'，请尝试使用双参数版本：
        // var BufferType = Java.use("android.widget.TextView$BufferType");
        // var editableBufferType = BufferType.EDITABLE.value;
        // codeInputField.setText.overload('java.lang.CharSequence', 'android.widget.TextView$BufferType').call(codeInputField, StringJava.$new(verificationCode), editableBufferType);
        console.log("[+] 验证码 '" + verificationCode + "' 已输入。期待自动提交。");
    };
}
// hook_bi();
// hook_binder_transact()
// hookHandleBindApplication();

},{"../../my_modules/common":1}],9:[function(require,module,exports){
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const hook_register_args_1 = require("./hook_register_args");
const ssl_1 = require("../../my_modules/ssl");
rpc.exports = {
    register: hook_register_args_1.register_request_verification_code,
    upload: hook_register_args_1.register_verification_code,
};
function ssl_callback(msg) {
    const body = msg.body;
    const ab = body?.buffer;
    msg.body = undefined;
    send(msg, ab);
}
(0, ssl_1.HookSSL)(ssl_callback, ssl_callback);

},{"../../my_modules/ssl":2,"./hook_register_args":8}]},{},[9])
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIm5vZGVfbW9kdWxlcy9icm93c2VyLXBhY2svX3ByZWx1ZGUuanMiLCJteV9tb2R1bGVzL2NvbW1vbi50cyIsIm15X21vZHVsZXMvc3NsLnRzIiwibXlfbW9kdWxlcy90cmFmZmljX2FuYWx5emVyLnRzIiwibm9kZV9tb2R1bGVzL2Jhc2U2NC1qcy9pbmRleC5qcyIsIm5vZGVfbW9kdWxlcy9mcmlkYS1idWZmZXIvaW5kZXguanMiLCJub2RlX21vZHVsZXMvZnJpZGEtYnVmZmVyL25vZGVfbW9kdWxlcy9idWZmZXIvaW5kZXguanMiLCJub2RlX21vZHVsZXMvaWVlZTc1NC9pbmRleC5qcyIsInByb2plY3RzL3dhL2hvb2tfcmVnaXN0ZXJfYXJncy50cyIsInByb2plY3RzL3dhL3JlZ2lzdGVyX3JwYy50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQTs7O0FDMmNBLGtDQW1EQztBQUVELGtDQTZDQztBQXdKRCxnQ0FxQkM7QUFFRCxvQ0FFQztBQUVELGdDQU9DO0FBR0Qsc0NBb0JDO0FBT0Qsb0NBMkJDO0FBTUQsOEJBc0NDO0FBRUQsZ0NBdURDO0FBOEJELG9EQTBEQztBQUVELHNEQTJCQztBQUVELG9EQWtEQztBQUVELG9CQWlCQztBQUVELG9DQUtDO0FBRUQsb0NBR0M7QUFZRCwwQ0FrQ0M7QUFFRCxnREFZQztBQU9ELDhDQW1DQztBQU9ELDBFQWNDO0FBRUQsMERBTUM7QUE2QkQsc0RBK0RDO0FBRUQsNERBTUM7QUFFRCxzQkFVQztBQUVELGtEQXNCQztBQUVELGdDQWdCQztBQUdELHdDQVVDO0FBT0Qsa0VBWUM7QUFHRCw0Q0FhQztBQU9ELHdDQWFDO0FBRUQsd0NBWUM7QUFrREQsNERBT0M7QUFHRCxzREFLQztBQUVELHdDQVdDO0FBRUQsa0NBaUJDO0FBRUQsOERBR0M7QUFFRCxvREEwQkM7QUFFRCxvQ0FtQkM7QUFFRCxvREFPQztBQUVELHdCQXFCQztBQU9ELGdDQVlDO0FBRUQsb0VBS0M7QUFPRCw4REFvQ0M7QUFFRCwwREFtQkM7QUFF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O2FBQU0sSUFBSSxDQUFDLElBQUksR0FBRyxJQUFJLENBQUMsS0FBSyxJQUFJLEVBQUUsQ0FBQztZQUNsQyxPQUFPLEdBQUcsQ0FBQyxTQUFTLENBQUMsR0FBRyxDQUFDLEdBQUcsR0FBRyxDQUFDLElBQUksRUFBRSxDQUFDO1lBQ3ZDLE9BQU8sSUFBSSxDQUFDLFNBQVMsQ0FBQyxHQUFHLEdBQUcsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1lBQzdDLE9BQU8sSUFBSSxDQUFDLFNBQVMsQ0FBQyxHQUFHLEdBQUcsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1lBQzdDLE9BQU8sSUFBSSxDQUFDLFNBQVMsQ0FBQyxHQUFHLEdBQUcsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1lBQzdDLE9BQU8sSUFBSSxDQUFDLFNBQVMsQ0FBQyxHQUFHLEdBQUcsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDO1lBQzVDLE9BQU8sSUFBSSxTQUFTLENBQUMsR0FBRyxHQUFHLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQztZQUNyQyxVQUFVLElBQUksTUFBTSxDQUFDLFlBQVksQ0FBQyxPQUFPLENBQUMsQ0FBQztZQUMzQyxHQUFHLElBQUksQ0FBQyxDQUFDO1FBQ1gsQ0FBQzthQUFNLElBQUksQ0FBQyxJQUFJLEdBQUcsSUFBSSxDQUFDLEtBQUssSUFBSSxFQUFFLENBQUM7WUFDbEMsT0FBTyxHQUFHLENBQUMsU0FBUyxDQUFDLEdBQUcsQ0FBQyxHQUFHLEdBQUcsQ0FBQyxJQUFJLEVBQUUsQ0FBQztZQUN2QyxPQUFPLElBQUksQ0FBQyxTQUFTLENBQUMsR0FBRyxHQUFHLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQyxJQUFJLEVBQUUsQ0FBQztZQUM3QyxPQUFPLElBQUksQ0FBQyxTQUFTLENBQUMsR0FBRyxHQUFHLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQyxJQUFJLEVBQUUsQ0FBQztZQUM3QyxPQUFPLElBQUksQ0FBQyxTQUFTLENBQUMsR0FBRyxHQUFHLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQztZQUM1QyxPQUFPLElBQUksU0FBUyxDQUFDLEdBQUcsR0FBRyxDQUFDLENBQUMsR0FBRyxJQUFJLENBQUM7WUFDckMsVUFBVSxJQUFJLE1BQU0sQ0FBQyxZQUFZLENBQUMsT0FBTyxDQUFDLENBQUM7WUFDM0MsR0FBRyxJQUFJLENBQUMsQ0FBQztRQUNYLENBQUM7YUFBTSxJQUFJLENBQUMsSUFBSSxHQUFHLElBQUksQ0FBQyxLQUFLLElBQUksRUFBRSxDQUFDO1lBQ2xDLE9BQU8sR0FBRyxDQUFDLFNBQVMsQ0FBQyxHQUFHLENBQUMsR0FBRyxHQUFHLENBQUMsSUFBSSxFQUFFLENBQUM7WUFDdkMsT0FBTyxJQUFJLENBQUMsU0FBUyxDQUFDLEdBQUcsR0FBRyxDQUFDLENBQUMsR0FBRyxJQUFJLENBQUMsSUFBSSxFQUFFLENBQUM7WUFDN0MsT0FBTyxJQUFJLENBQUMsU0FBUyxDQUFDLEdBQUcsR0FBRyxDQUFDLENBQUMsR0FBRyxJQUFJLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDNUMsT0FBTyxJQUFJLFNBQVMsQ0FBQyxHQUFHLEdBQUcsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDO1lBQ3JDLFVBQVUsSUFBSSxNQUFNLENBQUMsWUFBWSxDQUFDLE9BQU8sQ0FBQyxDQUFDO1lBQzNDLEdBQUcsSUFBSSxDQUFDLENBQUM7UUFDWCxDQUFDO2FBQU0sSUFBSSxDQUFDLElBQUksR0FBRyxJQUFJLENBQUMsS0FBSyxJQUFJLEVBQUUsQ0FBQztZQUNsQyxPQUFPLEdBQUcsQ0FBQyxTQUFTLENBQUMsR0FBRyxDQUFDLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1lBQ3hDLE9BQU8sSUFBSSxDQUFDLFNBQVMsQ0FBQyxHQUFHLEdBQUcsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDO1lBQzVDLE9BQU8sSUFBSSxTQUFTLENBQUMsR0FBRyxHQUFHLENBQUMs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QTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBOzs7QUN0SkE7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTs7Ozs7QUNQQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7Ozs7QUN6eERBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUM1RUEsMEJBMERDO0FBSUQsb0RBa0hDO0FBSUQsOERBb0NDO0FBNkJELGdGQU1DO0FBRUQsZ0VBSUM7QUExUUQsZ0VBQWtEO0FBRWxEOzs7OzZDQUk2QztBQUU3QyxJQUFJLE9BQU8sR0FBVyxDQUFDLENBQUMsQ0FBQztBQUN6QixTQUFnQixPQUFPO0lBQ3JCLHFDQUFxQztJQUNyQyxJQUFJLElBQUksR0FBRyxNQUFNLENBQUMsZ0JBQWdCLENBQUMsU0FBUyxFQUFFLE9BQU8sQ0FBQyxDQUFDO0lBQ3ZELElBQUksVUFBVSxHQUFHLHNDQUFzQyxDQUFDO0lBRXhELElBQUksSUFBSSxFQUFFLENBQUM7UUFDVCxXQUFXLENBQUMsTUFBTSxDQUFDLElBQUksRUFBRTtZQUN2QixPQUFPLEVBQUUsVUFBVSxJQUFJO2dCQUNyQixJQUFJLElBQUksR0FBRyxJQUFJLENBQUMsQ0FBQyxDQUFDLENBQUMsY0FBYyxFQUFFLENBQUM7Z0JBQ3BDLElBQUksSUFBSSxLQUFLLGlDQUFpQyxFQUFFLENBQUM7b0JBQy9DLGdDQUFnQztvQkFDaEMsSUFBSSxDQUFDLE1BQU0sR0FBRyxJQUFJLENBQUM7b0JBQ25CLE9BQU8sQ0FBQyxHQUFHLENBQUMseUJBQXlCLENBQUMsQ0FBQztnQkFDekMsQ0FBQztZQUNILENBQUM7WUFFRCxPQUFPLEVBQUUsVUFBVSxNQUFNO2dCQUN2QixJQUFJLElBQUksQ0FBQyxNQUFNLEVBQUUsQ0FBQztvQkFDaEIsNENBQTRDO29CQUM1QyxPQUFPLEdBQUcsTUFBTSxDQUFDLE1BQU0sQ0FBQyxDQUFDO29CQUN6QixPQUFPLENBQUMsR0FBRyxDQUFDLHdCQUF3QixFQUFFLE9BQU8sQ0FBQyxDQUFDO2dCQUNqRCxDQUFDO1lBQ0gsQ0FBQztTQUNGLENBQUMsQ0FBQztJQUNMLENBQUM7U0FBTSxDQUFDO1FBQ04sT0FBTyxDQUFDLEdBQUcsQ0FBQywwQkFBMEIsQ0FBQyxDQUFDO0lBQzFDLENBQUM7SUFFRCxxREFBcUQ7SUFDckQsK0RBQStEO0lBQy9ELElBQUksU0FBUyxHQUFHLE1BQU0sQ0FBQyxnQkFBZ0IsQ0FBQyxTQUFTLEVBQUUsT0FBTyxDQUFDLENBQUM7SUFDNUQsSUFBSSxTQUFTLEVBQUUsQ0FBQztRQUNkLFdBQVcsQ0FBQyxNQUFNLENBQUMsU0FBUyxFQUFFO1lBQzVCLE9BQU8sRUFBRSxVQUFVLElBQUk7Z0JBQ3JCLElBQUksQ0FBQyxNQUFNLEdBQUcsSUFBSSxDQUFDLENBQUMsQ0FBQyxDQUFDO2dCQUN0QixJQUFJLENBQUMsSUFBSSxHQUFHLElBQUksQ0FBQyxDQUFDLENBQUMsQ0FBQyxPQUFPLEVBQUUsQ0FBQztnQkFDOUIsSUFBSSxDQUFDLENBQUMsR0FBRyxJQUFJLENBQUMsQ0FBQyxDQUFDLENBQUMsT0FBTyxFQUFFLENBQUM7Z0JBQzNCLElBQUksQ0FBQyxFQUFFLEdBQUcsSUFBSSxDQUFDLENBQUMsQ0FBQyxDQUFDO1lBQ3BCLENBQUM7WUFDRCxPQUFPLEVBQUUsVUFBVSxNQUFNO2dCQUN2QixJQUFJLE1BQU0sQ0FBQyxJQUFJLENBQUMsRUFBRSxDQUFDLElBQUksT0FBTyxFQUFFLENBQUM7b0JBQy9CLDJGQUEyRjtvQkFDM0YsSUFBSSxPQUFPLEdBQUcsTUFBTSxDQUFDLElBQUksRUFBRSxHQUFHLElBQUksQ0FBQztvQkFDbkMsT0FBTyxDQUFDLEdBQUcsQ0FBQywwQkFBMEIsR0FBRyxPQUFPLENBQUMsQ0FBQztvQkFDbEQsSUFBSSxhQUFhLEdBQUcsT0FBTyxDQUFDLE1BQU0sQ0FBQztvQkFDbkMsSUFBSSxXQUFXLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyxhQUFhLEVBQUUsSUFBSSxDQUFDLElBQUksR0FBRyxJQUFJLENBQUMsQ0FBQyxDQUFDLENBQUM7b0JBQzlELElBQUksQ0FBQyxNQUFNLENBQUMsZUFBZSxDQUFDLE9BQU8sQ0FBQyxDQUFDO29CQUVyQyxPQUFPLENBQUMsR0FBRyxDQUFDLGNBQWMsT0FBTyxDQUFDLElBQUksQ0FBQyxNQUFNLEVBQUUsRUFBRSxNQUFNLEVBQUUsTUFBTSxDQUFDLE1BQU0sQ0FBQyxFQUFFLENBQUMsRUFBRSxDQUFDLENBQUM7b0JBQzlFLElBQUksWUFBWSxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsV0FBVyxHQUFHLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQztvQkFDdkQsTUFBTSxDQUFDLE9BQU8sQ0FBQyxHQUFHLENBQUMsWUFBWSxDQUFDLENBQUMsQ0FBQztvQkFDbEMsT0FBTyxHQUFHLENBQUMsQ0FBQyxDQUFDO2dCQUNmLENBQUM7WUFDSCxDQUFDO1NBQ0YsQ0FBQyxDQUFDO0lBQ0wsQ0FBQztTQUFNLENBQUM7UUFDTixPQUFPLENBQUMsR0FBRyxDQUFDLDBCQUEwQixDQUFDLENBQUM7SUFDMUMsQ0FBQztBQUNILENBQUM7QUFFRCxJQUFJLGFBQWEsR0FBRyw4QkFBOEIsQ0FBQztBQUNuRCxJQUFJLGtCQUFrQixHQUFHLENBQUMsQ0FBQztBQUMzQixTQUFnQixvQkFBb0I7SUFDbEMsSUFBSSxDQUFDLE9BQU8sQ0FBQztRQUNYLDRFQUE0RTtRQUM1RSxJQUFJLGdCQUFnQixHQUFHLFVBQVUsQ0FBQyxDQUFDLHNEQUFzRDtRQUN6RixJQUFJLGNBQWMsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLDhCQUE4QixDQUFDLENBQUM7UUFDOUQsSUFBSSxhQUFhLEdBQUcsSUFBSSxDQUFDLEtBQUssQ0FBQyw4QkFBOEIsRUFBRSxDQUFDLGNBQWMsQ0FBQyxJQUFJLENBQUMsZ0JBQWdCLENBQUMsQ0FBQyxDQUFDLENBQUM7UUFFeEcsMkJBQTJCO1FBQzNCLElBQUksY0FBYyxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsNEJBQTRCLENBQUMsQ0FBQztRQUM1RCxJQUFJLEdBQUcsR0FBRyxjQUFjLENBQUMsa0JBQWtCLEVBQUUsQ0FBQztRQUM5Qyw2Q0FBNkM7UUFDN0MsaURBQWlEO1FBQ2pELElBQUksY0FBYyxHQUFHLGNBQWMsQ0FBQztRQUVwQyxVQUFVO1FBQ1YsSUFBSSxXQUFXLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyx3QkFBd0IsQ0FBQyxDQUFDO1FBQ3JELElBQUksTUFBTSxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsbUJBQW1CLENBQUMsQ0FBQztRQUMzQyxJQUFJLFdBQVcsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLGdDQUFnQyxDQUFDLENBQUM7UUFDN0QsSUFBSSxLQUFLLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyxrQkFBa0IsQ0FBQyxDQUFDO1FBQ3pDLElBQUksYUFBYSxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsMEJBQTBCLENBQUMsQ0FBQztRQUN6RCxJQUFJLGVBQWUsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLG9DQUFvQyxDQUFDLENBQUM7UUFFckUsWUFBWTtRQUNaLElBQUkscUJBQXFCLEdBQUcsVUFBVSxDQUFDLENBQUMsZ0NBQWdDO1FBQ3hFLElBQUksMEJBQTBCLEdBQUcsQ0FBQyxDQUFDLENBQUMsMkZBQTJGO1FBQy9ILElBQUksOEJBQThCLEdBQUcsQ0FBQyxDQUFDLENBQUMsYUFBYTtRQUVyRCxnQkFBZ0I7UUFDaEIsV0FBVyxDQUFDLFFBQVEsQ0FBQyxjQUFjLEdBQUcsVUFBVSxJQUFTLEVBQUUsSUFBUyxFQUFFLEtBQVUsRUFBRSxLQUFVO1lBQzFGLElBQUksTUFBTSxHQUFHLElBQUksQ0FBQyxRQUFRLENBQUMsSUFBSSxFQUFFLElBQUksRUFBRSxLQUFLLEVBQUUsS0FBSyxDQUFDLENBQUM7WUFDckQsSUFBSSxJQUFJLEtBQUssSUFBSSxJQUFJLEtBQUssS0FBSyxJQUFJLEVBQUUsQ0FBQztnQkFDcEMsT0FBTyxNQUFNLENBQUM7WUFDaEIsQ0FBQztZQUNELElBQUksSUFBSSxLQUFLLHFCQUFxQixFQUFFLENBQUM7Z0JBQ25DLE9BQU8sTUFBTSxDQUFDO1lBQ2hCLENBQUM7WUFDRCxJQUFJLElBQUksR0FBRyxJQUFJLENBQUMsc0JBQXNCLEVBQUUsQ0FBQztZQUN6QyxJQUFJLElBQUksS0FBSyxJQUFJLElBQUksSUFBSSxLQUFLLG9DQUFvQyxFQUFFLENBQUM7Z0JBQ25FLE9BQU8sTUFBTSxDQUFDO1lBQ2hCLENBQUM7WUFDRCxJQUFJLElBQUksS0FBSywwQkFBMEIsRUFBRSxDQUFDO2dCQUN4QyxLQUFLLENBQUMsYUFBYSxFQUFFLENBQUM7Z0JBQ3RCLElBQUksS0FBSyxDQUFDLE9BQU8sRUFBRSxLQUFLLENBQUMsRUFBRSxDQUFDO29CQUMxQixJQUFJLFdBQVcsR0FBRyxXQUFXLENBQUMsT0FBTyxDQUFDLEtBQUssQ0FBQyxnQkFBZ0IsQ0FBQyxLQUFLLENBQUMsQ0FBQztvQkFDcEUsV0FBVyxHQUFHLElBQUksQ0FBQyxJQUFJLENBQUMsV0FBVyxFQUFFLFdBQVcsQ0FBQyxDQUFDO29CQUNsRCxPQUFPLENBQUMsR0FBRyxDQUFDLGVBQWUsR0FBRyxJQUFJLENBQUMsU0FBUyxDQUFDLFdBQVcsQ0FBQyxHQUFHLGlCQUFpQixHQUFHLFdBQVcsQ0FBQyxXQUFXLENBQUMsS0FBSyxDQUFDLENBQUM7b0JBQy9HLElBQUksTUFBTSxDQUFDLFdBQVcsQ0FBQyxXQUFXLENBQUMsS0FBSyxDQUFDLElBQUksY0FBYyxFQUFFLENBQUM7d0JBQzVELElBQUksYUFBYSxDQUFDLENBQUMsQ0FBQyxLQUFLLElBQUksRUFBRSxDQUFDOzRCQUM5QixPQUFPLENBQUMsS0FBSyxDQUFDLGtEQUFrRCxDQUFDLENBQUM7NEJBQ2xFLElBQUksQ0FBQyxHQUFHLENBQUMsb0JBQW9CLENBQUMsQ0FBQyxXQUFXLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxvQkFBb0IsQ0FBQyxDQUFDLEtBQUssRUFBRSxDQUFDLENBQUM7NEJBQ25GLE9BQU8sTUFBTSxDQUFDO3dCQUNoQixDQUFDO3dCQUVELElBQUksV0FBVyxDQUFDLFVBQVUsQ0FBQyxLQUFLLEtBQUssSUFBSSxJQUFJLFdBQVcsQ0FBQyxVQUFVLENBQUMsS0FBSyxDQUFDLE1BQU0sR0FBRyxDQUFDLEVBQUUsQ0FBQzs0QkFDckYsc0RBQXNEOzRCQUN0RCxPQUFPLENBQUMsR0FBRyxDQUFDLGdEQUFnRCxHQUFHLGFBQWEsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDO3dCQUNuRixDQUFDO3dCQUVELElBQUksYUFBYSxDQUFDLE9BQU8sSUFBSSxFQUFFLEVBQUUsQ0FBQzs0QkFDaEMsd0JBQXdCOzRCQUN4QixJQUFJLFdBQVcsQ0FBQyxXQUFXLENBQUMsS0FBSyxLQUFLLElBQUksRUFBRSxDQUFDO2dDQUMzQyxJQUFJLFdBQVcsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLGdDQUFnQyxDQUFDLENBQUM7Z0NBQzdELElBQUksZUFBZSxHQUFHLFdBQVcsQ0FBQyxXQUFXLENBQUMsS0FBSyxDQUFDLHFCQUFxQixFQUFFLENBQUM7Z0NBQzVFLElBQUksZUFBZSxLQUFLLElBQUksSUFBSSxlQUFlLENBQUMsTUFBTSxHQUFHLENBQUMsRUFBRSxDQUFDO29DQUMzRCx5Q0FBeUM7b0NBQ3pDLE9BQU8sQ0FBQyxHQUFHLENBQUMsZ0RBQWdELEdBQUcsYUFBYSxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUM7Z0NBQ25GLENBQUM7NEJBQ0gsQ0FBQzt3QkFDSCxDQUFDO3dCQUVELEtBQUssQ0FBQyxlQUFlLENBQUMsQ0FBQyxDQUFDLENBQUM7d0JBQ3pCLEtBQUssQ0FBQyxXQUFXLENBQUMsQ0FBQyxDQUFDLENBQUM7d0JBQ3JCLEtBQUssQ0FBQyxnQkFBZ0IsRUFBRSxDQUFDO3dCQUN6QixLQUFLLENBQUMsUUFBUSxDQUFDLENBQUMsQ0FBQyxDQUFDO3dCQUNsQixXQUFXLENBQUMsYUFBYSxDQUFDLEtBQUssRUFBRSxDQUFDLENBQUMsQ0FBQyxDQUFDLG9DQUFvQztvQkFDM0UsQ0FBQztnQkFDSCxDQUFDO2dCQUNELEtBQUssQ0FBQyxlQUFlLENBQUMsQ0FBQyxDQUFDLENBQUM7WUFDM0IsQ0FBQztpQkFBTSxJQUFJLElBQUksS0FBSyw4QkFBOEIsRUFBRSxDQUFDO2dCQUNuRCxLQUFLLENBQUMsYUFBYSxFQUFFLENBQUM7Z0JBQ3RCLElBQUksS0FBSyxDQUFDLE9BQU8sRUFBRSxLQUFLLENBQUMsRUFBRSxDQUFDO29CQUMxQixRQUFRO29CQUNSLDJGQUEyRjtvQkFDM0YsSUFBSSxPQUFPLEdBQUcsZUFBZSxDQUFDLE9BQU8sQ0FBQyxLQUFLLENBQUMsZ0JBQWdCLENBQUMsS0FBSyxDQUFDLENBQUM7b0JBQ3BFLHFEQUFxRDtvQkFDckQsT0FBTyxHQUFHLElBQUksQ0FBQyxJQUFJLENBQUMsT0FBTyxFQUFFLGVBQWUsQ0FBQyxDQUFDO29CQUM5QyxPQUFPLENBQUMsR0FBRyxDQUFDLFdBQVcsR0FBRyxJQUFJLENBQUMsU0FBUyxDQUFDLE9BQU8sQ0FBQyxHQUFHLGlCQUFpQixHQUFHLE9BQU8sQ0FBQyxXQUFXLENBQUMsS0FBSyxDQUFDLENBQUM7b0JBQ25HLElBQUksTUFBTSxDQUFDLE9BQU8sQ0FBQyxXQUFXLENBQUMsS0FBSyxDQUFDLElBQUksY0FBYyxFQUFFLENBQUM7d0JBQ3hELGVBQWU7d0JBQ2YsSUFBSSxPQUFPLEdBQUcsdUNBQXVDLEdBQUcsT0FBTyxDQUFDLFNBQVMsQ0FBQyxLQUFLLEdBQUcsSUFBSSxDQUFDO3dCQUN2RixPQUFPLENBQUMsU0FBUyxDQUFDLEtBQUssR0FBRyxhQUFhLENBQUM7d0JBQ3hDLE9BQU8sSUFBSSw4Q0FBOEMsR0FBRyxhQUFhLEdBQUcsSUFBSSxDQUFDO3dCQUNqRixPQUFPLElBQUksbUNBQW1DLEdBQUcsTUFBTSxDQUFDLHVCQUF1QixFQUFFLENBQUM7d0JBRWxGLE9BQU8sQ0FBQyxHQUFHLENBQUMsT0FBTyxDQUFDLENBQUM7d0JBRXJCLHdCQUF3Qjt3QkFDeEIsT0FBTyxDQUFDLGVBQWUsQ0FBQyxLQUFLLEdBQUcsYUFBYSxDQUFDO3dCQUU5QyxZQUFZO3dCQUNaLEtBQUssQ0FBQyxlQUFlLENBQUMsQ0FBQyxDQUFDLENBQUM7d0JBQ3pCLEtBQUssQ0FBQyxXQUFXLENBQUMsQ0FBQyxDQUFDLENBQUM7d0JBQ3JCLEtBQUssQ0FBQyxnQkFBZ0IsRUFBRSxDQUFDO3dCQUN6QixLQUFLLENBQUMsUUFBUSxDQUFDLENBQUMsQ0FBQyxDQUFDO3dCQUNsQixPQUFPLENBQUMsYUFBYSxDQUFDLEtBQUssRUFBRSxDQUFDLENBQUMsQ0FBQyxDQUFDLG9DQUFvQzt3QkFDckUsYUFBYSxJQUFJLEdBQUcsR0FBRyxNQUFNLENBQUMsa0JBQWtCLENBQUMsQ0FBQzt3QkFDbEQsa0JBQWtCLElBQUksQ0FBQyxDQUFDO29CQUMxQixDQUFDO2dCQUNILENBQUM7Z0JBQ0QsS0FBSyxDQUFDLGVBQWUsQ0FBQyxDQUFDLENBQUMsQ0FBQztZQUMzQixDQUFDO1lBQ0QsT0FBTyxNQUFNLENBQUM7UUFDaEIsQ0FBQyxDQUFDO0lBQ0osQ0FBQyxDQUFDLENBQUM7QUFDTCxDQUFDO0FBRUQsd0VBQXdFO0FBQ3hFLG9DQUFvQztBQUNwQyxTQUFnQix5QkFBeUI7SUFDdkMsNkJBQTZCO0lBQzdCLElBQUksYUFBYSxHQUFHLDhCQUE4QixDQUFDLENBQUMsaUJBQWlCO0lBRXJFLElBQUksY0FBYyxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsNEJBQTRCLENBQUMsQ0FBQztJQUU1RCx3REFBd0Q7SUFDeEQsY0FBYyxDQUFDLHFCQUFxQixDQUFDLGNBQWMsR0FBRyxVQUFVLElBQVM7UUFDdkUsNkJBQTZCO1FBQzdCLElBQUksTUFBTSxHQUFHLElBQUksQ0FBQyxxQkFBcUIsQ0FBQyxJQUFJLENBQUMsQ0FBQztRQUU5Qyx3REFBd0Q7UUFDeEQsSUFBSSxRQUFRLEdBQUcsSUFBSSxDQUFDLGlCQUFpQixDQUFDLEtBQUssQ0FBQztRQUM1QyxJQUFJLFFBQVEsS0FBSyxJQUFJLEVBQUUsQ0FBQztZQUN0QixJQUFJLFNBQVMsR0FBRyxRQUFRLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQztZQUNwQyxJQUFJLFNBQVMsS0FBSyxJQUFJLEVBQUUsQ0FBQztnQkFDdkIsSUFBSSxPQUFPLEdBQUcsU0FBUyxDQUFDLGdCQUFnQixDQUFDLEtBQUssQ0FBQztnQkFDL0MsSUFBSSxPQUFPLEtBQUssSUFBSSxFQUFFLENBQUM7b0JBQ3JCLElBQUksaUJBQWlCLEdBQUcsT0FBTyxDQUFDLFNBQVMsQ0FBQyxLQUFLLENBQUM7b0JBQ2hELE9BQU8sQ0FBQyxTQUFTLENBQUMsS0FBSyxHQUFHLGFBQWEsQ0FBQztvQkFDeEMsT0FBTyxDQUFDLGVBQWUsQ0FBQyxLQUFLLEdBQUcsYUFBYSxDQUFDLENBQUMsV0FBVztvQkFDMUQsT0FBTyxDQUFDLEdBQUcsQ0FBQyxvQ0FBb0MsR0FBRyxpQkFBaUIsR0FBRyxtQkFBbUIsR0FBRyxhQUFhLENBQUMsQ0FBQztvQkFDNUcsT0FBTyxDQUFDLEdBQUcsQ0FBQyw0QkFBNEIsR0FBRyxNQUFNLENBQUMsdUJBQXVCLEVBQUUsQ0FBQyxDQUFDO2dCQUMvRSxDQUFDO3FCQUFNLENBQUM7b0JBQ04sT0FBTyxDQUFDLEtBQUssQ0FBQywrQkFBK0IsQ0FBQyxDQUFDO2dCQUNqRCxDQUFDO1lBQ0gsQ0FBQztpQkFBTSxDQUFDO2dCQUNOLE9BQU8sQ0FBQyxLQUFLLENBQUMsaUNBQWlDLENBQUMsQ0FBQztZQUNuRCxDQUFDO1FBQ0gsQ0FBQzthQUFNLENBQUM7WUFDTixPQUFPLENBQUMsS0FBSyxDQUFDLGdDQUFnQyxDQUFDLENBQUM7UUFDbEQsQ0FBQztRQUVELE9BQU8sTUFBTSxDQUFDO0lBQ2hCLENBQUMsQ0FBQztJQUNGLE1BQU07QUFDUixDQUFDO0FBRUQsU0FBUyxTQUFTO0lBQ2hCLElBQUksSUFBSSxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsT0FBTyxDQUFDLENBQUM7SUFDN0IsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDLGNBQWMsR0FBRyxVQUFVLElBQVMsRUFBRSxLQUFVO1FBQzFELE9BQU8sQ0FBQyxHQUFHLENBQUMsOEJBQThCLE1BQU0sQ0FBQyxlQUFlLENBQUMsSUFBSSxDQUFDLGFBQWEsTUFBTSxDQUFDLGVBQWUsQ0FBQyxLQUFLLENBQUMsRUFBRSxDQUFDLENBQUM7UUFDcEgsSUFBSSxNQUFNLEdBQUcsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDLElBQUksRUFBRSxLQUFLLENBQUMsQ0FBQztRQUN0QyxPQUFPLENBQUMsR0FBRyxDQUFDLHFCQUFxQixNQUFNLENBQUMsZUFBZSxDQUFDLE1BQU0sQ0FBQyxFQUFFLENBQUMsQ0FBQztRQUNuRSxPQUFPLE1BQU0sQ0FBQztJQUNoQixDQUFDLENBQUM7SUFFRixJQUFJLElBQUksR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLE9BQU8sQ0FBQyxDQUFDO0lBQzdCLElBQUksQ0FBQyxLQUFLLENBQUMsQ0FBQyxjQUFjLEdBQUcsVUFBVSxJQUFTLEVBQUUsR0FBUSxFQUFFLElBQVM7UUFDbkUsT0FBTyxDQUFDLEdBQUcsQ0FBQyw0QkFBNEIsSUFBSSxTQUFTLEdBQUcsVUFBVSxJQUFJLEVBQUUsQ0FBQyxDQUFDO1FBQzFFLElBQUksTUFBTSxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsQ0FBQyxJQUFJLEVBQUUsR0FBRyxFQUFFLElBQUksQ0FBQyxDQUFDO1FBQzFDLE9BQU8sQ0FBQyxHQUFHLENBQUMsbUJBQW1CLE1BQU0sQ0FBQyxlQUFlLENBQUMsTUFBTSxDQUFDLEVBQUUsQ0FBQyxDQUFDO1FBQ2pFLE9BQU8sTUFBTSxDQUFDO0lBQ2hCLENBQUMsQ0FBQztBQUNKLENBQUM7QUFFRCxTQUFTLG9CQUFvQjtJQUMzQixJQUFJLENBQUMsT0FBTyxDQUFDO1FBQ1gsd0JBQXdCLENBQUMsUUFBUSxDQUFDLENBQUM7UUFDbkMsc0JBQXNCLEVBQUUsQ0FBQztRQUN6QixZQUFZLENBQUMsR0FBRyxFQUFFLFlBQVksQ0FBQyxDQUFDO1FBQ2hDLFNBQVMsRUFBRSxDQUFDO0lBQ2QsQ0FBQyxDQUFDLENBQUM7QUFDTCxDQUFDO0FBRUQsU0FBZ0Isa0NBQWtDLENBQUMsWUFBb0IsRUFBRSxZQUFvQjtJQUMzRixJQUFJLENBQUMsT0FBTyxDQUFDO1FBQ1gsc0JBQXNCLEVBQUUsQ0FBQztRQUN6QixZQUFZLENBQUMsWUFBWSxFQUFFLFlBQVksQ0FBQyxDQUFDO1FBQ3pDLFNBQVMsRUFBRSxDQUFDO0lBQ2QsQ0FBQyxDQUFDLENBQUM7QUFDTCxDQUFDO0FBRUQsU0FBZ0IsMEJBQTBCLENBQUMsSUFBWTtJQUNyRCxJQUFJLENBQUMsT0FBTyxDQUFDO1FBQ1gsd0JBQXdCLENBQUMsUUFBUSxDQUFDLENBQUM7SUFDckMsQ0FBQyxDQUFDLENBQUM7QUFDTCxDQUFDO0FBRUQsU0FBUyxTQUFTO0lBQ2hCLElBQUksVUFBVSxHQUFHLEtBQUssQ0FBQztJQUV2QixPQUFPLENBQUMsR0FBRyxDQUFDLDBDQUEwQyxDQUFDLENBQUM7SUFFeEQsSUFBSSxJQUFJLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyxvQ0FBb0MsQ0FBQyxDQUFDO0lBRTFELDZDQUE2QztJQUM3QyxJQUFJLENBQUMsb0JBQW9CLENBQUMsUUFBUSxDQUFDLFNBQVMsQ0FBQyxDQUFDLGNBQWMsR0FBRyxVQUFVLFFBQWE7UUFDcEYsY0FBYztRQUNkLElBQUksQ0FBQyxvQkFBb0IsQ0FBQyxRQUFRLENBQUMsQ0FBQztRQUVwQyxzQ0FBc0M7UUFDdEMsSUFBSSxDQUFDLFFBQVEsSUFBSSxVQUFVLEVBQUUsQ0FBQztZQUM1QixPQUFPO1FBQ1QsQ0FBQztRQUVELE9BQU8sQ0FBQyxHQUFHLENBQUMsc0RBQXNELENBQUMsQ0FBQztRQUVwRSxlQUFlO1FBQ2YsVUFBVSxHQUFHLElBQUksQ0FBQztRQUVsQixJQUFJLFlBQVksR0FBRyxJQUFJLENBQUMsQ0FBQyw2QkFBNkI7UUFFdEQsMkJBQTJCO1FBQzNCLElBQUksQ0FBQztZQUNILElBQUksV0FBVyxHQUFHLFlBQVksQ0FBQyxjQUFjLEVBQUUsQ0FBQztZQUNoRCxJQUFJLGtCQUFrQixHQUFHLFlBQVksQ0FBQyxZQUFZLEVBQUUsQ0FBQyxhQUFhLENBQUMsYUFBYSxFQUFFLElBQUksRUFBRSxXQUFXLENBQUMsQ0FBQztZQUVyRyxJQUFJLGtCQUFrQixLQUFLLENBQUMsRUFBRSxDQUFDO2dCQUM3QixPQUFPLENBQUMsR0FBRyxDQUFDLCtCQUErQixHQUFHLGtCQUFrQixDQUFDLENBQUM7Z0JBQ2xFLElBQUksV0FBVyxHQUFHLFlBQVksQ0FBQyxZQUFZLENBQUMsa0JBQWtCLENBQUMsQ0FBQztnQkFFaEUsSUFBSSxXQUFXLEVBQUUsQ0FBQztvQkFDaEIsT0FBTyxDQUFDLEdBQUcsQ0FBQyw0Q0FBNEMsQ0FBQyxDQUFDO29CQUMxRCxXQUFXLENBQUMsWUFBWSxFQUFFLENBQUM7b0JBQzNCLE9BQU8sQ0FBQyxHQUFHLENBQUMsMkJBQTJCLENBQUMsQ0FBQztnQkFDM0MsQ0FBQztxQkFBTSxDQUFDO29CQUNOLE9BQU8sQ0FBQyxHQUFHLENBQUMseUNBQXlDLENBQUMsQ0FBQztnQkFDekQsQ0FBQztZQUNILENBQUM7aUJBQU0sQ0FBQztnQkFDTixPQUFPLENBQUMsR0FBRyxDQUFDLG1DQUFtQyxDQUFDLENBQUM7WUFDbkQsQ0FBQztRQUNILENBQUM7UUFBQyxPQUFPLENBQU0sRUFBRSxDQUFDO1lBQ2hCLE9BQU8sQ0FBQyxHQUFHLENBQUMsb0JBQW9CLEdBQUcsQ0FBQyxDQUFDLE9BQU8sQ0FBQyxDQUFDO1FBQ2hELENBQUM7SUFDSCxDQUFDLENBQUM7QUFDSixDQUFDO0FBRUQsU0FBUyxZQUFZLENBQUMsWUFBb0IsRUFBRSxZQUFvQjtJQUM5RCxJQUFJLFlBQVksR0FBRyxLQUFLLENBQUM7SUFDekIsSUFBSSxhQUFhLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyw4REFBOEQsQ0FBQyxDQUFDO0lBQzdGLElBQUkseUJBQXlCLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyxPQUFPLENBQUMsQ0FBQztJQUNsRCxJQUFJLE1BQU0sR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLGtCQUFrQixDQUFDLENBQUM7SUFFMUMsYUFBYSxDQUFDLFFBQVEsQ0FBQyxjQUFjLEdBQUc7UUFDdEMsbURBQW1EO1FBQ25ELElBQUksQ0FBQyxRQUFRLEVBQUUsQ0FBQztRQUVoQixJQUFJLENBQUMsYUFBYSxDQUFDLEtBQUssQ0FBQyxVQUFVLENBQUMsSUFBSSxDQUFDLEVBQUUsQ0FBQztZQUMxQyxPQUFPLENBQUMsR0FBRyxDQUFDLHlCQUF5QixHQUFHLElBQUksQ0FBQyxRQUFRLEVBQUUsQ0FBQyxPQUFPLEVBQUUsR0FBRyxPQUFPLENBQUMsQ0FBQztZQUM3RSxPQUFPO1FBQ1QsQ0FBQztRQUVELElBQUksWUFBWSxFQUFFLENBQUM7WUFDakIsT0FBTyxDQUFDLEdBQUcsQ0FBQyxrQkFBa0IsQ0FBQyxDQUFDO1lBQ2hDLE9BQU87UUFDVCxDQUFDO1FBRUQsWUFBWSxHQUFHLElBQUksQ0FBQztRQUNwQixJQUFJLFFBQVEsR0FBRyxJQUFJLENBQUM7UUFFcEIsMENBQTBDO1FBQzFDLDhCQUE4QjtRQUM5Qiw2REFBNkQ7UUFDN0QsSUFBSSxlQUFlLEdBQUcsSUFBSSxDQUFDLElBQUksQ0FBQyxRQUFRLEVBQUUseUJBQXlCLENBQUMsQ0FBQztRQUNyRSw4REFBOEQ7UUFDOUQsSUFBSSxZQUFZLEdBQUcsZUFBZSxDQUFDLEdBQUcsQ0FBQyxLQUFLLENBQUM7UUFDN0MsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDO1lBQ2xCLE9BQU8sQ0FBQyxHQUFHLENBQUMsaUNBQWlDLENBQUMsQ0FBQztZQUMvQyxPQUFPO1FBQ1QsQ0FBQztRQUVELDRDQUE0QztRQUM1QyxJQUFJLG9CQUFvQixHQUFHLFlBQVksQ0FBQyxHQUFHLENBQUMsS0FBSyxDQUFDO1FBQ2xELElBQUksQ0FBQyxvQkFBb0IsRUFBRSxDQUFDO1lBQzFCLE9BQU8sQ0FBQyxHQUFHLENBQUMsc0RBQXNELENBQUMsQ0FBQztZQUNwRSxPQUFPO1FBQ1QsQ0FBQztRQUNELE9BQU8sQ0FBQyxHQUFHLENBQUMsZ0NBQWdDLENBQUMsQ0FBQztRQUU5QyxJQUFJLFdBQVcsR0FBRyxRQUFRLENBQUMsY0FBYyxFQUFFLENBQUM7UUFFNUMsaURBQWlEO1FBQ2pELElBQUksS0FBSyxHQUFHLFFBQVEsQ0FBQyxZQUFZLEVBQUUsQ0FBQyxhQUFhLENBQUMsaUJBQWlCLEVBQUUsSUFBSSxFQUFFLFdBQVcsQ0FBQyxDQUFDO1FBQ3hGLElBQUksUUFBUSxHQUFHLFFBQVEsQ0FBQyxZQUFZLEVBQUUsQ0FBQyxhQUFhLENBQUMsb0JBQW9CLEVBQUUsSUFBSSxFQUFFLFdBQVcsQ0FBQyxDQUFDO1FBRTlGLDJDQUEyQztRQUMzQyxJQUFJLG1CQUFtQixHQUFHLG9CQUFvQixDQUFDLFlBQVksQ0FBQyxLQUFLLENBQUMsQ0FBQztRQUNuRSxJQUFJLG1CQUFtQixHQUFHLG9CQUFvQixDQUFDLFlBQVksQ0FBQyxRQUFRLENBQUMsQ0FBQztRQUV0RSxJQUFJLENBQUMsbUJBQW1CLElBQUksQ0FBQyxtQkFBbUIsRUFBRSxDQUFDO1lBQ2pELE9BQU8sQ0FBQyxHQUFHLENBQUMseUVBQXlFLENBQUMsQ0FBQztZQUN2RixPQUFPO1FBQ1QsQ0FBQztRQUNELE9BQU8sQ0FBQyxHQUFHLENBQUMsK0JBQStCLENBQUMsQ0FBQztRQUU3QyxVQUFVO1FBQ1YseUVBQXlFO1FBQ3pFLG1CQUFtQixHQUFHLElBQUksQ0FBQyxJQUFJLENBQUMsbUJBQW1CLEVBQUUsSUFBSSxDQUFDLEdBQUcsQ0FBQyx5QkFBeUIsQ0FBQyxDQUFDLENBQUM7UUFDMUYsbUJBQW1CLEdBQUcsSUFBSSxDQUFDLElBQUksQ0FBQyxtQkFBbUIsRUFBRSxJQUFJLENBQUMsR0FBRyxDQUFDLHlCQUF5QixDQUFDLENBQUMsQ0FBQztRQUUxRixtQkFBbUIsQ0FBQyxPQUFPLENBQUMsUUFBUSxDQUFDLHdCQUF3QixDQUFDLENBQUMsSUFBSSxDQUFDLG1CQUFtQixFQUFFLE1BQU0sQ0FBQyxJQUFJLENBQUMsWUFBWSxDQUFDLENBQUMsQ0FBQztRQUNwSCxtQkFBbUIsQ0FBQyxPQUFPLENBQUMsUUFBUSxDQUFDLHdCQUF3QixDQUFDLENBQUMsSUFBSSxDQUFDLG1CQUFtQixFQUFFLE1BQU0sQ0FBQyxJQUFJLENBQUMsWUFBWSxDQUFDLENBQUMsQ0FBQztRQUNwSCxPQUFPLENBQUMsR0FBRyxDQUFDLHFCQUFxQixDQUFDLENBQUM7UUFFbkMsc0NBQXNDO1FBQ3RDLElBQUksY0FBYyxHQUFHLFFBQVEsQ0FBQyxZQUFZLEVBQUUsQ0FBQyxhQUFhLENBQUMscUJBQXFCLEVBQUUsSUFBSSxFQUFFLFdBQVcsQ0FBQyxDQUFDO1FBQ3JHLElBQUksY0FBYyxFQUFFLENBQUM7WUFDbkIsSUFBSSxZQUFZLEdBQUcsUUFBUSxDQUFDLFlBQVksQ0FBQyxjQUFjLENBQUMsQ0FBQztZQUN6RCxJQUFJLFlBQVksRUFBRSxDQUFDO2dCQUNqQixZQUFZLENBQUMsWUFBWSxFQUFFLENBQUM7Z0JBQzVCLE9BQU8sQ0FBQyxHQUFHLENBQUMsNEJBQTRCLENBQUMsQ0FBQztZQUM1QyxDQUFDO2lCQUFNLENBQUM7Z0JBQ04sT0FBTyxDQUFDLEdBQUcsQ0FBQyx3QkFBd0IsQ0FBQyxDQUFDO1lBQ3hDLENBQUM7UUFDSCxDQUFDO2FBQU0sQ0FBQztZQUNOLE9BQU8sQ0FBQyxHQUFHLENBQUMsMkNBQTJDLENBQUMsQ0FBQztRQUMzRCxDQUFDO1FBQ0QsTUFBTTtJQUNSLENBQUMsQ0FBQztBQUNKLENBQUM7QUFFRDs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztHQXlCRztBQUNILFNBQVMsc0JBQXNCO0lBQzdCLElBQUksYUFBYSxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsOERBQThELENBQUMsQ0FBQztJQUM3RixJQUFJLFFBQVEsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLHNCQUFzQixDQUFDLENBQUM7SUFDaEQsUUFBUSxDQUFDLFVBQVUsQ0FBQyxRQUFRLENBQUMsS0FBSyxDQUFDLENBQUMsY0FBYyxHQUFHLFVBQVUsRUFBTztRQUNwRSxrREFBa0Q7UUFDbEQsSUFBSSxhQUFhLENBQUMsS0FBSyxDQUFDLFVBQVUsQ0FBQyxJQUFJLENBQUMsSUFBSSxFQUFFLEtBQUssRUFBRSxFQUFFLENBQUM7WUFDdEQsT0FBTyxDQUFDLEdBQUcsQ0FBQyxtQ0FBbUMsQ0FBQyxDQUFDO1lBRWpELHdCQUF3QjtZQUN4QixhQUFhLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxDQUFDO1lBQ3hCLE9BQU8sQ0FBQyxHQUFHLENBQUMsMEJBQTBCLENBQUMsQ0FBQztZQUV4Qyx3Q0FBd0M7WUFDeEMsT0FBTztRQUNULENBQUM7UUFFRCxpQkFBaUI7UUFDakIsT0FBTyxJQUFJLENBQUMsVUFBVSxDQUFDLEVBQUUsQ0FBQyxDQUFDO0lBQzdCLENBQUMsQ0FBQztBQUNKLENBQUM7QUFFRCxTQUFTLG1CQUFtQjtJQUMxQiwwQ0FBMEM7SUFFMUMsSUFBSSxNQUFNLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyxvQkFBb0IsQ0FBQyxDQUFDO0lBQzVDLElBQUksZUFBZSxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsaUNBQWlDLENBQUMsQ0FBQztJQUNsRSw2QkFBNkI7SUFFN0IsTUFBTSxDQUFDLElBQUksQ0FBQyxjQUFjLEdBQUc7UUFDM0IsaUJBQWlCO1FBQ2pCLElBQUksQ0FBQyxJQUFJLEVBQUUsQ0FBQztRQUNaLDZCQUE2QjtRQUU3QixJQUFJLE1BQU0sR0FBRyxJQUFJLENBQUM7UUFDbEIsT0FBTyxDQUFDLEdBQUcsQ0FBQyw0QkFBNEIsQ0FBQyxDQUFDO1FBRTFDLDBDQUEwQztRQUMxQyx3Q0FBd0M7UUFDeEMsSUFBSSxnQkFBZ0IsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLGNBQWMsQ0FBQyxDQUFDLE9BQU8sQ0FBQyxLQUFLLENBQUM7UUFDOUQsSUFBSSxjQUFjLEdBQUcsTUFBTSxDQUFDLFlBQVksQ0FBQyxnQkFBZ0IsQ0FBQyxDQUFDO1FBQzNELGNBQWMsR0FBRyxJQUFJLENBQUMsSUFBSSxDQUFDLGNBQWMsRUFBRSxJQUFJLENBQUMsR0FBRyxDQUFDLHVCQUF1QixDQUFDLENBQUMsQ0FBQztRQUM5RSxvRUFBb0U7UUFFcEUsSUFBSSxjQUFjLEVBQUUsQ0FBQztZQUNuQixtQ0FBbUM7WUFDbkMsT0FBTyxDQUFDLEdBQUcsQ0FBQyw2QkFBNkIsQ0FBQyxDQUFDO1lBQzNDLGNBQWMsQ0FBQyxZQUFZLEVBQUUsQ0FBQztZQUM5QixPQUFPLENBQUMsR0FBRyxDQUFDLHlCQUF5QixDQUFDLENBQUM7UUFDekMsQ0FBQztRQUNELE1BQU07SUFDUixDQUFDLENBQUM7QUFDSixDQUFDO0FBRUQsU0FBUyx3QkFBd0IsQ0FBQyxnQkFBd0I7SUFDeEQsSUFBSSx1QkFBdUIsR0FBRyxLQUFLLENBQUM7SUFDcEMsMkNBQTJDO0lBQzNDLElBQUksVUFBVSxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsa0JBQWtCLENBQUMsQ0FBQztJQUM5QyxJQUFJLGlCQUFpQixHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsNkRBQTZELENBQUMsQ0FBQztJQUNoRyxJQUFJLGNBQWMsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLDZCQUE2QixDQUFDLENBQUM7SUFDN0QsSUFBSSxZQUFZLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyx3QkFBd0IsQ0FBQyxDQUFDLENBQUMsaUVBQWlFO0lBQ3hILDhFQUE4RTtJQUM5RSxrREFBa0Q7SUFDbEQseUhBQXlIO0lBRXpILGlCQUFpQixDQUFDLFFBQVEsQ0FBQyxjQUFjLEdBQUc7UUFDMUMsSUFBSSxDQUFDLFFBQVEsRUFBRSxDQUFDLENBQUMsaUJBQWlCO1FBQ2xDLElBQUksdUJBQXVCO1lBQUUsT0FBTztRQUVwQyxPQUFPO1FBQ1AsSUFBSSxDQUFDLGlCQUFpQixDQUFDLEtBQUssQ0FBQyxVQUFVLENBQUMsSUFBSSxDQUFDLEVBQUUsQ0FBQztZQUM5QyxPQUFPLENBQUMsR0FBRyxDQUFDLG9DQUFvQyxHQUFHLElBQUksQ0FBQyxRQUFRLEVBQUUsQ0FBQyxPQUFPLEVBQUUsR0FBRyxNQUFNLENBQUMsQ0FBQztZQUN2RixPQUFPO1FBQ1QsQ0FBQztRQUVELHVCQUF1QixHQUFHLElBQUksQ0FBQztRQUMvQixPQUFPLENBQUMsR0FBRyxDQUFDLCtDQUErQyxDQUFDLENBQUM7UUFDN0QsSUFBSSxRQUFRLEdBQUcsSUFBSSxDQUFDO1FBRXBCLElBQUksY0FBYyxHQUFHLFFBQVEsQ0FBQyxHQUFHLENBQUMsS0FBSyxDQUFDLENBQUMsdUJBQXVCO1FBQ2hFLElBQUksQ0FBQyxjQUFjLEVBQUUsQ0FBQztZQUNwQixPQUFPLENBQUMsR0FBRyxDQUFDLG1DQUFtQyxDQUFDLENBQUM7WUFDakQsT0FBTztRQUNULENBQUM7UUFFRCxlQUFlO1FBQ2YsbUNBQW1DO1FBRW5DLGdDQUFnQztRQUNoQyxjQUFjLEdBQUcsSUFBSSxDQUFDLElBQUksQ0FBQyxjQUFjLEVBQUUsSUFBSSxDQUFDLEdBQUcsQ0FBQyx5QkFBeUIsQ0FBQyxDQUFDLENBQUM7UUFDaEYsY0FBYyxDQUFDLE9BQU8sQ0FBQyxRQUFRLENBQUMsd0JBQXdCLENBQUMsQ0FBQyxJQUFJLENBQUMsY0FBYyxFQUFFLFVBQVUsQ0FBQyxJQUFJLENBQUMsZ0JBQWdCLENBQUMsQ0FBQyxDQUFDO1FBQ2xILGlEQUFpRDtRQUNqRCxtRUFBbUU7UUFDbkUsc0RBQXNEO1FBQ3RELCtLQUErSztRQUUvSyxPQUFPLENBQUMsR0FBRyxDQUFDLFdBQVcsR0FBRyxnQkFBZ0IsR0FBRyxlQUFlLENBQUMsQ0FBQztJQUNoRSxDQUFDLENBQUM7QUFDSixDQUFDO0FBRUQsYUFBYTtBQUNiLHlCQUF5QjtBQUV6QiwrQkFBK0I7Ozs7O0FDamhCL0IsNkRBQXNHO0FBQ3RHLDhDQUErQztBQUcvQyxHQUFHLENBQUMsT0FBTyxHQUFHO0lBQ1osUUFBUSxFQUFFLHVEQUFrQztJQUM1QyxNQUFNLEVBQUUsK0NBQTBCO0NBQ25DLENBQUM7QUFFRixTQUFTLFlBQVksQ0FBQyxHQUFnQjtJQUNwQyxNQUFNLElBQUksR0FBRyxHQUFHLENBQUMsSUFBSSxDQUFDO0lBQ3RCLE1BQU0sRUFBRSxHQUFHLElBQUksRUFBRSxNQUFxQixDQUFDO0lBQ3ZDLEdBQUcsQ0FBQyxJQUFJLEdBQUcsU0FBUyxDQUFDO0lBRXJCLElBQUksQ0FBQyxHQUFHLEVBQUUsRUFBRSxDQUFDLENBQUM7QUFDaEIsQ0FBQztBQUVELElBQUEsYUFBTyxFQUFDLFlBQVksRUFBRSxZQUFZLENBQUMsQ0FBQyIsImZpbGUiOiJnZW5lcmF0ZWQuanMiLCJzb3VyY2VSb290IjoiIn0=
