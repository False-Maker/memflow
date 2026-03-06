import keyring

services = ['openai', 'anthropic', 'embedding']
for svc in services:
    try:
        key = keyring.get_password('memflow', svc)
        if key:
            print(svc + ": 已保存 (length=" + str(len(key)) + ")")
        else:
            print(svc + ": 未保存")
    except Exception as e:
        print(svc + ": 错误 - " + str(e))
