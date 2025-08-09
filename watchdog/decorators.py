from database import Session


def with_db_session(func):
    """
    Decorator for automatically creating a session when running a function.
    """

    def wrapper(*args, **kwargs):
        session = Session()
        try:
            result = func(session, *args, **kwargs)
            session.commit()
            return result
        except:
            session.rollback()
            raise
        finally:
            session.close()

    return wrapper
