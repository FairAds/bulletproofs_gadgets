#[cfg(target_os="android")]
#[allow(non_snake_case)]

pub mod bulletproofs_android {
    extern crate jni;
    extern crate bulletproofs_gadgets;

    use std::convert::TryFrom;

    use jni::JNIEnv;
    use jni::objects::{JClass, JObject, JString, JValue};
    use jni::sys::{jbyteArray, jboolean};

    use bulletproofs_gadgets::prove::prove;
    use bulletproofs_gadgets::verify::verify;

    //
    // Signature for the BulletproofWrapper used in the Android side (output of javap -s)
    //

    /*
    Compiled from "RustBulletproofs.java"
    class com.unholster.examplebulletproofs.BulletproofWrapper {

        java.lang.String name;
            descriptor: Ljava/lang/String;
        java.lang.String instance;
            descriptor: Ljava/lang/String;
        java.lang.String witness;
            descriptor: Ljava/lang/String;
        java.lang.String gadgets;
            descriptor: Ljava/lang/String;
        java.lang.String commitments;
            descriptor: Ljava/lang/String;
        byte[] proof;
            descriptor: [B

        public com.unholster.examplebulletproofs.BulletproofWrapper(java.lang.String, java.lang.String, java.lang.String, java.lang.String);
            descriptor: (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V
        public com.unholster.examplebulletproofs.BulletproofWrapper(java.lang.String, java.lang.String, java.lang.String, java.lang.String, byte[]);
            descriptor: (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;[B)V

        public java.lang.String getName();
            descriptor: ()Ljava/lang/String;
        public void setName(java.lang.String);
            descriptor: (Ljava/lang/String;)V
        public java.lang.String getInstance();
            descriptor: ()Ljava/lang/String;
        public void setInstance(java.lang.String);
            descriptor: (Ljava/lang/String;)V
        public java.lang.String getWitness();
            descriptor: ()Ljava/lang/String;
        public void setWitness(java.lang.String);
            descriptor: (Ljava/lang/String;)V
        public java.lang.String getGadgets();
            descriptor: ()Ljava/lang/String;
        public void setGadgets(java.lang.String);
            descriptor: (Ljava/lang/String;)V
        public java.lang.String getCommitments();
            descriptor: ()Ljava/lang/String;
        public void setCommitments(java.lang.String);
            descriptor: (Ljava/lang/String;)V
        public byte[] getProof();
            descriptor: ()[B
        public void setProof(byte[]);
            descriptor: ([B)V
    }
    */

    fn get_jobject_member_string(env: &JNIEnv, object: JObject, getter: &str, signature: &str) -> Result<String, Box<dyn std::error::Error>> {
        let jvalue: JValue = env.call_method(object, getter, signature, &[])?;
        let jstring: JString = jvalue.l()?.into();
        Ok(env.get_string(jstring)?.into())
    }

    fn get_jobject_member_bytes(env: &JNIEnv, object: JObject, getter: &str, signature: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let jvalue: JValue = env.call_method(object, getter, signature, &[])?;
        let jarray: jbyteArray = jvalue.l()?.into_inner();  // jbyteArray = jarray = jobject
        Ok(env.convert_byte_array(jarray)?)
    }

    #[no_mangle]
    pub unsafe extern fn Java_com_unholster_examplebulletproofs_RustBulletproofs_extProve(env: JNIEnv, _: JClass, data: JObject) {
        let name: String = get_jobject_member_string(&env, data, "getName", "()Ljava/lang/String;").unwrap();
        let instance: String = get_jobject_member_string(&env, data, "getInstance", "()Ljava/lang/String;").unwrap();
        let witness: String = get_jobject_member_string(&env, data, "getWitness", "()Ljava/lang/String;").unwrap();
        let gadgets: String = get_jobject_member_string(&env, data, "getGadgets", "()Ljava/lang/String;").unwrap();
        let mut commitments = String::new();
        let proof = prove(Box::leak(name.into_boxed_str()), instance, witness, gadgets, &mut commitments).expect("unable to generate proof from provided Android data");

        let java_commitments: JObject = env.new_string(commitments).unwrap().into();
        env.call_method(data, "setCommitments", "(Ljava/lang/String;)V", &[JValue::from(java_commitments)]).unwrap();
        let java_proof: JObject = JObject::from(env.byte_array_from_slice(&proof[..]).unwrap());
        env.call_method(data, "setProof", "([B)V", &[JValue::from(java_proof)]).unwrap();
    }

    #[no_mangle]
    pub unsafe extern fn Java_com_unholster_examplebulletproofs_RustBulletproofs_extVerify(env: JNIEnv, _: JClass, data: JObject) -> jboolean {
        let name: String = get_jobject_member_string(&env, data, "getName", "()Ljava/lang/String;").unwrap();
        let instance: String = get_jobject_member_string(&env, data, "getInstance", "()Ljava/lang/String;").unwrap();
        let commitments: String = get_jobject_member_string(&env, data, "getCommitments", "()Ljava/lang/String;").unwrap();
        let gadgets: String = get_jobject_member_string(&env, data, "getGadgets", "()Ljava/lang/String;").unwrap();
        let proof: Vec<u8> = get_jobject_member_bytes(&env, data, "getProof", "()[B").unwrap();

        let verified = verify(Box::leak(name.into_boxed_str()), instance, proof, commitments, gadgets).expect("unable to verify proof from provided Android data");
        jboolean::try_from(JValue::from(verified)).unwrap()
    }
}
