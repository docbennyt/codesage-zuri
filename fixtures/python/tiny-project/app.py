from auth import authenticate_user

def login(email, password):
    if not email:
        return False
    return authenticate_user(email, password)
