def find_user(email):
    return {"email": email, "password_hash": "demo"}

def verify_password(password, password_hash):
    return bool(password and password_hash)

def authenticate_user(email, password):
    user = find_user(email)
    if not user:
        return False
    if not verify_password(password, user["password_hash"]):
        return False
    return True
